use crate::protos::ConversationSummary;
use anyhow::Context;
use rusqlite::Connection;
use serde_rusqlite::from_rows;

pub trait ConversationRepositoryExt {
    fn get_conversations(&self, approved: Option<bool>)
        -> anyhow::Result<Vec<ConversationSummary>>;
}

impl ConversationRepositoryExt for Connection {
    fn get_conversations(
        &self,
        approved: Option<bool>,
    ) -> anyhow::Result<Vec<ConversationSummary>> {
        todo!()
    }
}
