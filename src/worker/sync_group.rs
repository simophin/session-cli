use crate::base64::Base64;
use crate::clock::ClockSource;
use crate::config::{
    Config, GroupConfig, GroupInfo, GroupInfoConfig, GroupMemberConfig, NamedConfig,
    SubaccountAuth, UserGroupsConfig,
};
use crate::config::{Group, GroupKeys};
use crate::db::config::ConfigRepositoryExt;
use crate::db::Repository;
use crate::ed25519::{ED25519PubKey, ED25519SecKey};
use crate::identity::Identity;
use crate::network::swarm::{SwarmAuth, SwarmState};
use crate::oxen_api::namespace::{
    GroupInfoConfigNamespace, GroupKeysNamespace, GroupMemberConfigNamespace, GroupNamespace,
};
use crate::oxen_api::JsonRpcCallSource;
use crate::session_id::{GroupID, IndividualID};
use anyhow::{anyhow, Context};
use futures_util::future::{select, select_all, Either};
use serde::Serialize;
use std::borrow::Cow;
use std::pin::{pin, Pin};
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use tokio::{select, try_join};

struct GroupSyncState<F> {
    group_id: GroupID,
    syncing_group: GroupInfo,
    configs: watch::Sender<GroupConfigState>,
    task: Pin<Box<F>>,
}

pub struct GroupConfigState {
    pub(super) group_id: GroupID,
    pub(super) group_keys: GroupKeys,
    pub(super) group_info: GroupInfoConfig,
    pub(super) group_members: GroupMemberConfig,
}

impl GroupConfigState {
    pub fn new(
        repo: &Repository,
        user_key: &ED25519SecKey,
        group_id: GroupID,
        group_admin_key: Option<&ED25519SecKey>,
    ) -> anyhow::Result<Self> {
        let mut group_info = GroupInfoConfig::new(
            &group_id,
            group_admin_key,
            Some(
                repo.get_config_dump(GroupInfoConfig::CONFIG_TYPE_NAME, Some(group_id.as_str()))
                    .context("Failed to get group info dump")?
                    .as_ref(),
            ),
        )
        .context("Failed to create group info config")?;

        let mut group_members = GroupMemberConfig::new(
            &group_id,
            group_admin_key,
            Some(
                repo.get_config_dump(GroupMemberConfig::CONFIG_TYPE_NAME, Some(group_id.as_str()))
                    .context("Failed to get group members dump")?
                    .as_ref(),
            ),
        )
        .context("Failed to create group members config")?;

        let group_keys = GroupKeys::new(
            user_key,
            group_id.pub_key(),
            group_admin_key,
            &mut group_info,
            &mut group_members,
            repo.get_config_dump(GroupKeys::CONFIG_TYPE_NAME, Some(group_id.as_str()))
                .context("Failed to get group members dump")?
                .as_ref(),
        )
        .context("Error creating group keys")?;

        Ok(Self {
            group_id,
            group_keys,
            group_info,
            group_members,
        })
    }
}

pub async fn sync_groups<CS>(
    call_source: &CS,
    identity: &Identity,
    repo: &Repository,
    mut config: watch::Receiver<UserGroupsConfig>,
    manual_sync_trigger: broadcast::Receiver<()>,
    clock: &ClockSource,
) -> anyhow::Result<()>
where
    CS: for<'a> JsonRpcCallSource<SourceArg<'a> = SwarmState>,
{
    let mut group_sync_states: Vec<GroupSyncState<_>> = Default::default();
    let mut update_group_futures = true;

    loop {
        if update_group_futures {
            log::info!("Updating group tasks");

            let groups: Vec<_> = config
                .borrow()
                .get_groups()
                .filter_map(|s| match s {
                    Group::Group(g) => Some(g),
                    _ => None,
                })
                .collect();

            let mut group_ids = groups
                .iter()
                .map(|g| g.group_id().unwrap())
                .collect::<Vec<_>>();

            group_ids.sort();

            // Go through the future list and spawn new future when necessary
            for group in groups {
                let group_id = group.group_id().unwrap();
                match group_sync_states.binary_search_by_key(&&group_id, |s| &s.group_id) {
                    Ok(index) if group_sync_states[index].syncing_group != group => {
                        log::info!("Group changed, restarting sync for group {:?}", group_id);
                        let mut sync_state = &mut group_sync_states[index];
                        sync_state.task = Box::pin(sync_group(
                            call_source,
                            repo,
                            group,
                            sync_state.configs.clone(),
                            config.clone(),
                            manual_sync_trigger.resubscribe(),
                            clock,
                        ));
                    }

                    Ok(_) => {
                        // Do nothing as no update
                    }

                    Err(index) => {
                        log::info!("New group found, starting sync for group {:?}", group_id);
                        let configs = watch::channel(GroupConfigState::new(
                            repo,
                            identity.ed25519_sec_key(),
                            group_id.clone(),
                            group.sec_key().as_ref(),
                        )?)
                        .0;

                        group_sync_states.insert(
                            index,
                            GroupSyncState {
                                group_id,
                                syncing_group: group.clone(),
                                configs: configs.clone(),
                                task: Box::pin(sync_group(
                                    call_source,
                                    repo,
                                    group,
                                    configs,
                                    config.clone(),
                                    manual_sync_trigger.resubscribe(),
                                    clock,
                                )),
                            },
                        );
                    }
                }
            }

            // Remove all the group futures that are not in the group_ids list
            group_sync_states.retain(|state| group_ids.contains(&state.group_id));
            update_group_futures = false
        }

        if group_sync_states.is_empty() {
            log::info!("No groups to sync. Waiting for config change");
            let _ = config.changed().await;
            update_group_futures = true;
            continue;
        }

        let drive_all_group_futures = select_all(
            group_sync_states.iter_mut().map(|s| s.task.as_mut()), // Add a pending future to make sure there's something to select
        );
        let drive_all_group_futures = pin!(drive_all_group_futures);
        let config_changed = pin!(config.changed());

        match select(config_changed, drive_all_group_futures).await {
            Either::Left((Ok(_), _)) => {
                update_group_futures = true;
            }
            Either::Left((Err(_), _)) => return Ok(()),
            Either::Right(((result, index, _), _)) => {
                if let Err(e) = result {
                    log::error!("Error while syncing group: {:?}. Stop syncing", e);
                }

                group_sync_states.remove(index);
            }
        }
    }
}

async fn sync_group<CS>(
    call_source: &CS,
    repo: &Repository,
    group: GroupInfo,
    group_config_state: watch::Sender<GroupConfigState>,
    user_groups_state: watch::Receiver<UserGroupsConfig>,
    manual_sync_trigger: broadcast::Receiver<()>,
    clock: &ClockSource,
) -> anyhow::Result<()>
where
    CS: for<'a> JsonRpcCallSource<SourceArg<'a> = SwarmState>,
{
    let group_id = group.group_id().context("Invalid group id in the info")?;
    let group_swarm = SwarmState::new(group_id.clone().into());
    let group_auth = (group_config_state.subscribe(), user_groups_state);

    let poll = super::sync_messages::<GroupNamespace, _>(
        repo,
        call_source,
        group_swarm.clone(),
        &group_auth,
        Duration::from_secs(10),
        manual_sync_trigger.resubscribe(),
        clock,
    );

    let (info_tx, info_rx) = tokio::sync::mpsc::channel(10);
    let (members_tx, members_rx) = tokio::sync::mpsc::channel(10);
    let (keys_tx, mut keys_rx) = tokio::sync::mpsc::channel(10);

    let poll_info = super::stream_messages::<GroupInfoConfigNamespace, _>(
        &info_tx,
        call_source,
        group_swarm.clone(),
        repo,
        clock,
        &group_auth,
        Duration::from_secs(10),
        manual_sync_trigger.resubscribe(),
    );

    let poll_members = super::stream_messages::<GroupMemberConfigNamespace, _>(
        &members_tx,
        call_source,
        group_swarm.clone(),
        repo,
        clock,
        &group_auth,
        Duration::from_secs(10),
        manual_sync_trigger.resubscribe(),
    );

    let mut poll_keys = pin!(super::stream_messages::<GroupKeysNamespace, _>(
        &keys_tx,
        call_source,
        group_swarm.clone(),
        repo,
        clock,
        &group_auth,
        Duration::from_secs(10),
        manual_sync_trigger.resubscribe(),
    ));

    let messages = select! {
        r = &mut poll_keys => return r,
        first_key_message = keys_rx.recv() => first_key_message,
    };

    match messages {
        Some(Ok(messages)) if !messages.is_empty() => {
            log::info!(
                "Loading initial {} group config keys messages",
                messages.len()
            );

            group_config_state.send_if_modified(|state| {
                let _ = state
                    .group_keys
                    .merge(&messages, (&mut state.group_info, &mut state.group_members));
                true
            });
        }
        _ => {}
    }

    let sync_configs = super::sync_group_configs::save_group_configs(
        repo,
        &group_config_state,
        info_rx,
        keys_rx,
        members_rx,
    );

    try_join!(poll, poll_info, poll_members, poll_keys, sync_configs)?;
    Ok(())
}

#[derive(Serialize)]
#[serde(untagged)]
enum GroupSignature {
    AdminSignature { signature: Base64<[u8; 64]> },
    MemberSignature(SubaccountAuth),
}

impl SwarmAuth
    for (
        watch::Receiver<GroupConfigState>,
        watch::Receiver<UserGroupsConfig>,
    )
{
    type SessionIDType = GroupID;

    fn sign(&self, payload: &[u8]) -> Option<impl Serialize + 'static> {
        let group_configs = self.0.borrow();
        let user_groups = self.1.borrow();

        let group_info = user_groups
            .get_groups()
            .filter_map(|g| match g {
                Group::Group(info) if info.group_id().as_ref() == Some(&group_configs.group_id) => {
                    Some(info)
                }
                _ => None,
            })
            .next()?;

        if let Some(sec_key) = group_info.sec_key() {
            return Some(GroupSignature::AdminSignature {
                signature: Base64(sec_key.sign(payload)),
            });
        } else if let Some(auth_data) = group_info.auth_data() {
            match group_configs.group_keys.sub_key_sign(payload, auth_data) {
                Err(e) => {
                    log::error!("Error signing message: {e:?}");
                    return None;
                }
                Ok(data) => Some(GroupSignature::MemberSignature(data)),
            }
        } else {
            log::error!(
                "No admin key or auth data found for group {:?}",
                group_configs.group_id
            );
            None
        }
    }

    fn decrypt(
        &self,
        payload: &[u8],
    ) -> anyhow::Result<(IndividualID, impl AsRef<[u8]> + 'static)> {
        let (sender, plaintext) = self
            .0
            .borrow()
            .group_keys
            .decrypt_message(payload)
            .context("Error decrypting message")?;

        let sender = sender
            .try_into()
            .map_err(|_| anyhow!("Invalid session ID"))?;
        Ok((sender, plaintext))
    }

    fn session_id(&self) -> Cow<Self::SessionIDType> {
        Cow::Owned(self.0.borrow().group_id.clone())
    }

    fn ed25519_pub_key(&self) -> Cow<ED25519PubKey> {
        Cow::Owned(self.session_id().pub_key().clone())
    }
}
