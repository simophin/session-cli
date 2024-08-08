use crate::clock::Timestamp;
use crate::db::models::MessageSource;
use crate::oxenss::namespace::MessageNamespace;
use crate::session_id::{IndividualOrBlindedID, SessionID};
use anyhow::Context;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_rusqlite::to_params_named;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MessageJobState {
    None,
    PendingRemove,
    PendingSend,
    FailedRemove,
    FailedSend,
}

#[derive(Serialize)]
pub struct Message<'a> {
    pub source: &'a MessageSource<'a>,
    pub hash: Option<&'a str>,
    pub content: Cow<'a, str>,
    pub sender: Cow<'a, IndividualOrBlindedID>,
    pub receiver: Cow<'a, SessionID>,
    pub created_at: Timestamp,
    pub expiration_at: Timestamp,
    pub quoting_timestamp: Option<u64>,
    pub job_state: MessageJobState,
}

pub trait MessageRepositoryExt {
    fn save_messages<'a>(&self, messages: impl Iterator<Item = Message<'a>>) -> anyhow::Result<()>;

    fn save_last_message_hash<NS: MessageNamespace>(
        &self,
        source: &MessageSource<'_>,
        last_hash: &str,
    ) -> anyhow::Result<()>;
    fn get_last_message_hash<NS: MessageNamespace>(
        &self,
        source: &MessageSource<'_>,
    ) -> anyhow::Result<Option<String>>;
}

impl MessageRepositoryExt for Connection {
    fn save_messages<'a>(&self, messages: impl Iterator<Item = Message<'a>>) -> anyhow::Result<()> {
        let mut stmt = self.prepare_cached("INSERT OR IGNORE INTO \
            messages(source, hash, content, sender, receiver, created_at, expiration_at, quoting_timestamp, job_state) \
            VALUES (:source, :hash, :content, :sender, :receiver, :created_at, :expiration_at, :quoting_timestamp, :job_state)").context("Prepare insert statement")?;

        for msg in messages {
            stmt.execute(
                to_params_named(msg)
                    .context("Serialising save message")?
                    .to_slice()
                    .as_slice(),
            )
            .context("Saving message")?;
        }

        Ok(())
    }

    fn save_last_message_hash<NS: MessageNamespace>(
        &self,
        source: &MessageSource<'_>,
        last_hash: &str,
    ) -> anyhow::Result<()> {
        self.execute(
            "INSERT OR REPLACE INTO message_retrieve_state(source, namespace, last_message_hash) VALUES (?, ?, ?)",
            params![source, NS::INT_VALUE, last_hash],
        )
        .context("Saving message retrieve state")?;

        Ok(())
    }

    fn get_last_message_hash<NS: MessageNamespace>(
        &self,
        source: &MessageSource<'_>,
    ) -> anyhow::Result<Option<String>> {
        self.query_row(
            "SELECT last_message_hash FROM message_retrieve_state WHERE source = ? AND namespace = ?",
            params![source, NS::INT_VALUE],
            |row| row.get(0),
        ).optional().context("Getting last message hash")
    }
}
