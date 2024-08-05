use anyhow::Context;
use rusqlite::{params, types::FromSql, Connection, ToSql};

use crate::app_setting::AppSetting;

pub trait AppSettingRepositoryExt {
    fn load_setting<T: AppSetting + FromSql>(&self, id: Option<&str>) -> anyhow::Result<T>;
    fn save_setting<T: AppSetting + ToSql>(
        &self,
        id: Option<&str>,
        value: &T,
    ) -> anyhow::Result<()>;
    fn remove_settings_by_name(&self, name: &str) -> anyhow::Result<()>;
}

impl AppSettingRepositoryExt for Connection {
    fn load_setting<T: AppSetting + FromSql>(&self, id: Option<&str>) -> anyhow::Result<T> {
        self.query_row(
            "SELECT value FROM app_settings WHERE name = ? AND id = ?",
            params![T::NAME, id.unwrap_or_default()],
            |row| row.get::<_, T>(0),
        )
        .context("Error getting setting")
    }

    fn save_setting<T: AppSetting + ToSql>(
        &self,
        id: Option<&str>,
        value: &T,
    ) -> anyhow::Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO app_settings (name, id, value) VALUES (?, ?, ?)",
            params![T::NAME, id.unwrap_or_default(), value],
        )
        .context("Error setting setting")?;

        Ok(())
    }

    fn remove_settings_by_name(&self, name: &str) -> anyhow::Result<()> {
        self.execute("DELETE FROM app_settings WHERE name = ?", params![name])
            .context("Error removing settings")?;
        Ok(())
    }
}
