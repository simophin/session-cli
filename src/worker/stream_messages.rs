use crate::clock::ClockSource;
use crate::db::messages::MessageRepositoryExt;
use crate::db::models::MessageSource;
use crate::db::Repository;
use crate::network::swarm::SwarmAuth;
use crate::oxenss::namespace::MessageNamespace;
use crate::oxenss::retrieve::{Message, RetrieveMessageRequest};
use crate::oxenss::{JsonRpcCallSource, JsonRpcCallSourceExt};
use anyhow::Context;
use std::time::Duration;
use tokio::select;
use tokio::sync::{broadcast, mpsc};

pub async fn stream_messages<NS, CS>(
    dest: &mpsc::Sender<anyhow::Result<Vec<Message>>>,
    call_source: &CS,
    call_source_arg: CS::SourceArg<'_>,
    repo: &Repository,
    clock: &ClockSource,
    swarm_auth: &impl SwarmAuth,
    poll_interval: Duration,
    mut manual_poll_trigger: broadcast::Receiver<()>,
) -> anyhow::Result<()>
where
    CS: JsonRpcCallSource,
    NS: MessageNamespace,
    for<'a> <CS as JsonRpcCallSource>::SourceArg<'a>: Clone,
{
    let message_source = MessageSource::from(swarm_auth.session_id().into_owned().into());
    log::info!(
        "Start streaming message for {message_source}, NS = {}",
        NS::DISPLAY_NAME
    );

    loop {
        let last_hash = repo
            .get_last_message_hash::<NS>(&message_source)
            .context("Retrieving latest hash")?;
        let resp = call_source
            .perform_json_rpc(
                call_source_arg.clone(),
                &RetrieveMessageRequest::<NS>::new(
                    swarm_auth,
                    last_hash.as_ref().map(|s| s.as_str()),
                    None,
                    clock.now_or_uncalibrated(),
                )?,
            )
            .await;

        match resp {
            Ok(mut resp) => {
                if let Some(last_hash) = resp.latest_hash() {
                    repo.save_last_message_hash::<NS>(&message_source, last_hash)
                        .context("Saving latest message hash")?;
                }

                resp.messages.sort_by(|a, b| a.created.cmp(&b.created));

                log::info!(
                    "Received {} new messages on {}",
                    resp.messages.len(),
                    NS::DISPLAY_NAME
                );

                if dest.send(Ok(resp.messages)).await.is_err() {
                    break;
                }
            }

            Err(e) => {
                log::error!("Error polling messages: {:?}", e);
                if dest.send(Err(e.into())).await.is_err() {
                    break;
                }
            }
        }

        select! {
            _ = tokio::time::sleep(poll_interval) => {
                log::debug!("{}s delay reached for pooling {}", poll_interval.as_secs(), NS::DISPLAY_NAME);
            }
            _ = manual_poll_trigger.recv() => {
                log::debug!("Manual trigger received for pooling {}", NS::DISPLAY_NAME);
            }
        }
    }

    log::info!(
        "Stop streaming message for {message_source}, NS = {}",
        NS::DISPLAY_NAME
    );
    Ok(())
}
