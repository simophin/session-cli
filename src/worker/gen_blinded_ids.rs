use tokio::sync::watch;

use crate::{
    app_setting::AppSetting,
    blinding::blinded_ids,
    config::{Group, UserGroupsConfig},
    db::{app_setting::AppSettingRepositoryExt, Repository},
    identity::Identity,
    network::swarm::SwarmAuth,
    session_id::BlindedID,
};

pub async fn gen_blinded_ids(
    identity: &Identity,
    mut config_watcher: watch::Receiver<UserGroupsConfig>,
    repo: &Repository,
) -> anyhow::Result<()> {
    loop {
        let conn = repo.obtain_connection()?;
        let tx = conn.unchecked_transaction()?;
        tx.remove_settings_by_name(<BlindedID as AppSetting>::NAME)?;

        let config = config_watcher.borrow();
        for g in config.get_groups().filter_map(|s| match s {
            Group::Community(c) => Some(c),
            _ => None,
        }) {
            let (blinded_id, _) =
                blinded_ids(identity.session_id().as_str(), &hex::encode(&g.pubkey))?;
            tx.save_setting(Some(&g.url_as_key()), &blinded_id)?;
        }

        drop(config);
        tx.commit()?;

        config_watcher.changed().await?;
    }
}
