use std::time::Duration;

use anyhow::Context;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::time::sleep;
use tokio::try_join;

use crate::clock::ClockSource;
use crate::config::Config;
use crate::db::config::ConfigRepositoryExt;
use crate::db::Repository;
use crate::network::swarm::SwarmAuth;
use crate::oxen_api::{namespace::MessageNamespace, JsonRpcCallSource, JsonRpcCallSourceExt};

pub async fn sync_config<NS, CS>(
    call_source: &CS,
    call_source_arg: CS::SourceArg<'_>,
    config_id: Option<&str>,
    watcher: &watch::Sender<impl for<'a> Config<MergeArg<'a> = ()>>,
    poll_interval: Duration,
    manual_trigger: broadcast::Receiver<()>,
    repo: &Repository,
    swarm_auth: &impl SwarmAuth,
    clock_source: &ClockSource,
) -> anyhow::Result<()>
where
    NS: MessageNamespace,
    CS: JsonRpcCallSource,
    for<'a> <CS as JsonRpcCallSource>::SourceArg<'a>: Clone,
{
    let (msg_tx, mut msg_rx) = mpsc::channel(10);

    let streaming = super::stream_messages::<NS, _>(
        &msg_tx,
        call_source,
        call_source_arg.clone(),
        repo,
        clock_source,
        swarm_auth,
        poll_interval,
        manual_trigger,
    );

    let streamed_to_config = async {
        while let Some(Ok(messages)) = msg_rx.recv().await {
            if messages.is_empty() {
                continue;
            }

            watcher.send_if_modified(|c| {
                if let Err(e) = c.merge(&messages, ()) {
                    log::error!("Error merging messages into config: {e:?}");
                }
                true
            });
        }
        Ok(())
    };

    let save = save_config_to_db::<NS, _>(watcher, repo, config_id);
    let push = push_config_if_needed::<NS, _>(call_source, call_source_arg, watcher, swarm_auth);

    try_join!(streaming, streamed_to_config, save, push)?;
    Ok(())
}

pub(super) async fn save_config_to_db<NS: MessageNamespace, C: Config>(
    config: &watch::Sender<C>,
    repo: &Repository,
    config_id: Option<&str>,
) -> anyhow::Result<()> {
    let mut config_rx = config.subscribe();
    loop {
        let mut err = None;
        config.send_if_modified(|config| {
            if let Err(e) = repo.save_config(config, config_id) {
                err.replace(e);
            }

            false
        });

        if let Some(e) = err {
            return Err(e);
        }

        config_rx.changed().await.context("Waiting for config")?;
    }
}

pub(super) async fn push_config_if_needed<NS, CS>(
    call_source: &CS,
    call_source_arg: CS::SourceArg<'_>,
    config: &watch::Sender<impl Config>,
    swarm_auth: &impl SwarmAuth,
) -> anyhow::Result<()>
where
    NS: MessageNamespace,
    CS: JsonRpcCallSource,
    for<'a> <CS as JsonRpcCallSource>::SourceArg<'a>: Clone,
{
    let mut config_rx = config.subscribe();

    while config_rx.wait_for(|c| c.needs_push()).await.is_ok() {
        let mut push_data = None;
        config.send_if_modified(|c| {
            push_data.replace(c.push());
            false
        });

        let push = async {
            let push_data = push_data
                .context("Empty push data")?
                .context("Error getting pushing data from config system")?;

            anyhow::Ok(())
        };

        if let Err(e) = push.await {
            let duration = Duration::from_secs(5);
            log::error!(
                "Failed to push config, wait for {}s before retrying: {e:?}",
                duration.as_secs()
            );
            let _ = sleep(duration).await;
        }
    }
    Ok(())
}
