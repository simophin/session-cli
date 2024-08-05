use crate::protos::{ConversationSummary, GetConversationsRequest, GetConversationsResponse};
use anyhow::Context;
use rusqlite::Connection;
use serde_rusqlite::from_rows;

pub trait ConversationRepositoryExt {
    fn get_conversations(
        &self,
        request: GetConversationsRequest,
    ) -> anyhow::Result<GetConversationsResponse>;
}

impl ConversationRepositoryExt for Connection {
    fn get_conversations(
        &self,
        _request: GetConversationsRequest,
    ) -> anyhow::Result<GetConversationsResponse> {
        let mut st = self
            .prepare_cached(include_str!("sql/get_conversations.sql"))
            .context("Preparing SQL")?;

        let conversations: Result<_, _> =
            from_rows::<ConversationSummary>(st.query([]).context("Run query")?)
                .into_iter()
                .collect();

        Ok(GetConversationsResponse {
            conversations: conversations.context("Deserializing conversation")?,
        })
    }
}
