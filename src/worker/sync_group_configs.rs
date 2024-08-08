use super::sync_group::GroupConfigState;
use crate::config::Config;
use crate::db::config::ConfigRepositoryExt;
use crate::db::Repository;
use crate::oxenss::retrieve::Message;
use tokio::select;
use tokio::sync::{mpsc, watch};

pub async fn save_group_configs(
    repo: &Repository,
    config: &watch::Sender<GroupConfigState>,
    mut group_info_config_messages: mpsc::Receiver<anyhow::Result<Vec<Message>>>,
    mut group_key_config_messages: mpsc::Receiver<anyhow::Result<Vec<Message>>>,
    mut group_members_config_messages: mpsc::Receiver<anyhow::Result<Vec<Message>>>,
) -> anyhow::Result<()> {
    loop {
        let (info, key, members) = select! {
            info = group_info_config_messages.recv() => {
                (info, group_key_config_messages.try_recv().ok(), group_members_config_messages.try_recv().ok())
            }
            key = group_key_config_messages.recv() => {
                (group_info_config_messages.try_recv().ok(), key, group_members_config_messages.try_recv().ok())
            }
            members = group_members_config_messages.recv() => {
                (group_info_config_messages.try_recv().ok(), group_key_config_messages.try_recv().ok(), members)
            }
        };

        if matches!(&info, Some(Ok(_)))
            || matches!(&key, Some(Ok(_)))
            || matches!(&members, Some(Ok(_)))
        {
            let mut err = None;
            config.send_if_modified(|state| {
                if let Some(Ok(messages)) = info {
                    let _ = state.group_info.merge(&messages, ());
                }

                if let Some(Ok(messages)) = members {
                    let _ = state.group_members.merge(&messages, ());
                }

                if let Some(Ok(messages)) = key {
                    let _ = state
                        .group_keys
                        .merge(&messages, (&mut state.group_info, &mut state.group_members));
                }

                if let Err(e) = state.save_to_db(repo) {
                    err.replace(e);
                }

                true
            });

            if let Some(e) = err {
                return Err(e);
            }
        } else if info.is_none() || key.is_none() || members.is_none() {
            return Ok(());
        }
    }
}

impl GroupConfigState {
    fn save_to_db(&mut self, repo: &Repository) -> anyhow::Result<()> {
        let tx = repo.begin_transaction()?;

        let group_id = self.group_id.as_str();
        tx.save_config(&mut self.group_info, Some(group_id))?;
        tx.save_config(&mut self.group_members, Some(group_id))?;
        tx.save_config(&mut self.group_keys, Some(group_id))?;

        tx.commit()?;
        Ok(())
    }
}
