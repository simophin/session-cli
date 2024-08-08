use std::borrow::Cow;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::{broadcast, mpsc};
use tokio::try_join;

use crate::clock::ClockSource;
use crate::db::messages::{Message as DbMessage, MessageJobState};
use crate::db::models::MessageSource;
use crate::db::{messages::MessageRepositoryExt, Repository};
use crate::network::swarm::SwarmAuth;
use crate::oxenss::message::{RegularMessage, RegularMessageDecoder};
use crate::oxenss::namespace::MessageNamespace;
use crate::oxenss::retrieve::Message as ApiMessage;
use crate::oxenss::JsonRpcCallSource;
use crate::session_id::SessionID;

pub async fn sync_messages<NS, CS>(
    repo: &Repository,
    call_source: &CS,
    call_source_arg: CS::SourceArg<'_>,
    swarm_auth: &impl SwarmAuth,
    poll_interval: Duration,
    manual_poll_trigger: broadcast::Receiver<()>,
    clock_source: &ClockSource,
) -> anyhow::Result<()>
where
    NS: MessageNamespace + RegularMessageDecoder,
    CS: JsonRpcCallSource,
    for<'a> <CS as JsonRpcCallSource>::SourceArg<'a>: Clone,
{
    let session_id: SessionID = swarm_auth.session_id().into_owned().into();
    let message_source: MessageSource = session_id.clone().into();

    let (message_tx, mut message_rx) = mpsc::channel(10);

    let stream = super::stream_messages::<NS, CS>(
        &message_tx,
        call_source,
        call_source_arg,
        repo,
        clock_source,
        swarm_auth,
        poll_interval,
        manual_poll_trigger,
    );

    let save = async {
        while let Some(Ok(messages)) = message_rx.recv().await {
            let tx = repo.begin_transaction().context("Starting transaction")?;
            tx.save_messages(messages.iter().filter_map(|msg| {
                create_db_message::<NS>(swarm_auth, &message_source, msg)
                    .inspect_err(|e| log::error!("Failed to create db message: {e:?}"))
                    .ok()
            }))
            .context("Saving messages")?;
            tx.commit().context("Committing transaction")?;
        }

        Ok(())
    };

    try_join!(stream, save)?;
    Ok(())
}

fn create_db_message<'a, Decoder: RegularMessageDecoder>(
    auth: &'a impl SwarmAuth,
    source: &'a MessageSource<'_>,
    ApiMessage {
        data,
        hash,
        expiration,
        created,
    }: &'a ApiMessage,
) -> anyhow::Result<DbMessage<'a>> {
    let RegularMessage { sender, content } = Decoder::decode_and_decrypt(data.as_slice(), auth)?;

    let receiver = match content
        .data_message
        .as_ref()
        .and_then(|d| d.sync_target.as_ref())
    {
        Some(c) => c.parse().context("Parsing sync_target as session ID")?,
        None => auth.session_id().into_owned().into(),
    };

    Ok(DbMessage {
        source,
        hash: Some(&hash),
        content: Cow::Owned(
            serde_json::to_string(&content).context("Serialising message content")?,
        ),
        sender: Cow::Owned(sender),
        receiver: Cow::Owned(receiver),
        created_at: *created,
        expiration_at: *expiration,
        quoting_timestamp: content
            .data_message
            .as_ref()
            .and_then(|s| s.quote.as_ref())
            .map(|q| q.id),
        job_state: MessageJobState::None,
    })
}
