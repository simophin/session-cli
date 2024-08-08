use anyhow::Context;
use tokio::sync::watch;
use tokio::try_join;

use crate::oxenss::namespace::{
    MessageNamespace, UserGroupsConfigNamespace, UserProfileConfigNamespace,
};

use crate::config::{
    Config, ContactsConfig, ConvoInfoVolatileConfig, IndividualConfig, UserGroupsConfig,
    UserProfileConfig,
};
use crate::db::config::ConfigRepositoryExt;
use crate::db::Repository;
use crate::ed25519::ED25519SecKey;

pub struct ConfigState {
    pub user_profile_config: watch::Sender<UserProfileConfig>,
    pub user_groups_config: watch::Sender<UserGroupsConfig>,
    pub convo_info_volatile_config: watch::Sender<ConvoInfoVolatileConfig>,
    pub contacts_config: watch::Sender<ContactsConfig>,
}

impl ConfigState {
    fn new_config_from_db_dump<C: IndividualConfig>(
        repo: &Repository,
        key: &ED25519SecKey,
    ) -> anyhow::Result<C> {
        let name = C::CONFIG_TYPE_NAME;
        let dump = repo
            .get_config_dump(name, None)
            .with_context(|| format!("Getting {name}'s dump from db"))?;

        C::new(key, Some(dump.as_slice()))
            .with_context(|| format!("Unable to create {name} from dump"))
    }

    pub fn new(repo: &Repository, sec_key: &ED25519SecKey) -> anyhow::Result<Self> {
        Ok(Self {
            user_profile_config: watch::channel(Self::new_config_from_db_dump(repo, sec_key)?).0,
            user_groups_config: watch::channel(Self::new_config_from_db_dump(repo, sec_key)?).0,
            convo_info_volatile_config: watch::channel(Self::new_config_from_db_dump(
                repo, sec_key,
            )?)
            .0,
            contacts_config: watch::channel(Self::new_config_from_db_dump(repo, sec_key)?).0,
        })
    }

    pub async fn log_configs(&self) -> anyhow::Result<()> {
        try_join!(
            Self::log_config(&self.user_profile_config),
            Self::log_config(&self.user_groups_config),
            Self::log_config(&self.convo_info_volatile_config),
            Self::log_config(&self.contacts_config)
        )?;
        Ok(())
    }

    async fn log_config<C: Config>(tx: &watch::Sender<C>) -> anyhow::Result<()> {
        let mut c = tx.subscribe();
        loop {
            let config = c.borrow();
            let name = config.config_type_name();
            let value: serde_json::Value = config.to_json().context("Config to json")?;
            let value = serde_json::to_string_pretty(&value).context("Formatting json")?;
            log::info!("{name} updated to: {value}");

            drop(config);

            c.changed().await.context("Waiting for config")?;
        }
    }
}
