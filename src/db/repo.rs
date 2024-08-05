use anyhow::Context;
use derive_more::Deref;
use rusqlite::hooks::Action;
use rusqlite::Transaction;
use tokio::sync::broadcast;

#[derive(Deref)]
pub struct Repository {
    #[deref]
    pub(super) db: rusqlite::Connection,
    table_change_broadcast: broadcast::Receiver<String>,
}

impl Repository {
    pub fn new(conn_str: &str) -> anyhow::Result<Self> {
        let mut db = rusqlite::Connection::open(conn_str)
            .with_context(|| format!("Error connecting to {conn_str}"))?;

        super::migrations::create_migrations()
            .to_latest(&mut db)
            .context("Error running db migrations")?;

        let (table_change_broadcast_tx, table_change_broadcast) = broadcast::channel(10);
        db.update_hook(Some(move |_: Action, _: &str, table: &str, _: i64| {
            let _ = table_change_broadcast_tx.send(table.to_string());
        }));

        Ok(Self {
            db,
            table_change_broadcast,
        })
    }

    pub fn begin_transaction(&self) -> rusqlite::Result<Transaction> {
        self.db.unchecked_transaction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_works() {
        let repo = Repository::new(":memory:").expect("To create repo");
    }
}
