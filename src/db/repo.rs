use anyhow::Context;
use derive_more::Deref;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::hooks::Action;
use std::ops::DerefMut;
use std::sync::Arc;
use strum::EnumString;
use tokio::sync::broadcast;

#[derive(Deref)]
pub struct Repository {
    #[deref]
    pub(super) db: Pool<SqliteConnectionManager>,

    table_change_broadcast: broadcast::Receiver<TableName>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TableName {
    Messages,
    Configs,
    AppSettings,
    Other(Arc<str>),
}

impl Repository {
    pub fn new(manager: SqliteConnectionManager) -> anyhow::Result<Self> {
        let db = Pool::new(manager)?;
        let mut conn = db.get()?;

        super::migrations::create_migrations()
            .to_latest(conn.deref_mut())
            .context("Error running db migrations")?;

        let (table_change_broadcast_tx, table_change_broadcast) = broadcast::channel(10);
        conn.update_hook(Some(move |_: Action, _: &str, table: &str, _: i64| {
            let _ = table_change_broadcast_tx
                .send(table.parse().unwrap_or(TableName::Other(table.into())));
        }));

        Ok(Self {
            db,
            table_change_broadcast,
        })
    }

    pub fn subscribe_table_changes(&self) -> broadcast::Receiver<TableName> {
        self.table_change_broadcast.resubscribe()
    }

    pub fn obtain_connection(
        &self,
    ) -> anyhow::Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.db.get().context("Getting pool connection")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_works() {
        let repo = Repository::new(SqliteConnectionManager::memory()).expect("To create repo");
    }
}
