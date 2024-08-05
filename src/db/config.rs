use crate::base64::Base64;
use crate::config::Config;
use anyhow::Context;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension};

pub trait ConfigRepositoryExt {
    fn save_config<C: Config>(&self, config: &mut C, id: Option<&str>) -> anyhow::Result<()>;
    fn save_config_raw(
        &self,
        config_type: &str,
        id: Option<&str>,
        value: &str,
        dump: Option<&[u8]>,
    ) -> anyhow::Result<()>;

    fn get_config_dump(&self, config_type: &str, id: Option<&str>) -> anyhow::Result<Vec<u8>>;
}

impl ConfigRepositoryExt for Connection {
    fn save_config<C: Config>(&self, config: &mut C, id: Option<&str>) -> anyhow::Result<()> {
        let dump = config.dump();
        self.save_config_raw(
            C::CONFIG_TYPE_NAME,
            id,
            &serde_json::to_string(&config.to_json()?)?,
            dump.as_ref().map(|d| d.as_ref()),
        )
    }

    fn save_config_raw(
        &self,
        config_type: &str,
        id: Option<&str>,
        value: &str,
        dump: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        let dump = dump.map(|d| BASE64_STANDARD.encode(d));

        self.execute(
            "INSERT OR REPLACE INTO configs(config_type, id, value, dump) VALUES (?, ?, ?, ?)",
            params![
                config_type,
                id.unwrap_or_default(),
                value,
                dump.unwrap_or_default()
            ],
        )?;

        Ok(())
    }

    fn get_config_dump(&self, config_type: &str, id: Option<&str>) -> anyhow::Result<Vec<u8>> {
        let dump = self
            .query_row(
                "SELECT dump FROM configs WHERE config_type = ? AND id = ?",
                params![config_type, id.unwrap_or_default()],
                |row| row.get::<_, Base64<Vec<u8>>>(0),
            )
            .optional()
            .context("Getting config dump")?;

        Ok(dump.map(|d| d.0).unwrap_or_default())
    }
}
