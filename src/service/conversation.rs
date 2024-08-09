use super::State;
use crate::db::conversations::ConversationRepositoryExt;
use crate::db::watch::with_changes;
use crate::db::TableName;
use crate::protos::{ListConversationsRequest, ListConversationsResponse};
use futures_core::Stream;
use std::time::Duration;

const WATCH_TABLES: &[TableName] = &[
    TableName::Configs,
    TableName::AppSettings,
    TableName::Messages,
];

pub async fn list<'a>(
    state: &'a State<'_>,
    req: ListConversationsRequest,
) -> impl Stream<Item = anyhow::Result<ListConversationsResponse>> + Send + Sync + 'a {
    let ListConversationsRequest { approved } = req;

    with_changes(
        state.repo.subscribe_table_changes(),
        WATCH_TABLES,
        Duration::from_secs(1),
        move || async move {
            let conversations = state
                .repo
                .obtain_connection()?
                .get_conversations(approved)?;

            let response = ListConversationsResponse {
                conversations: conversations.into_iter().map(Into::into).collect(),
            };

            Ok(response)
        },
    )
}
