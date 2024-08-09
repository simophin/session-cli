use crate::db::TableName;
use async_stream::stream;
use futures_core::Stream;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::sleep;

pub fn with_changes<'a, F>(
    mut receiver: broadcast::Receiver<TableName>,
    tables_to_watch: &'a [TableName],
    min_interval: Duration,
    mut produce: impl FnMut() -> F + 'a,
) -> impl Stream<Item = F::Output> + 'a
where
    F: Future,
    F::Output: 'a,
{
    stream! {
        loop {
            yield produce().await;

            if wait_for(&mut receiver, min_interval, tables_to_watch).await.is_none() {
                break;
            }
        }
    }
}

async fn wait_for(
    receiver: &mut broadcast::Receiver<TableName>,
    min_interval: Duration,
    tables_to_watch: &[TableName],
) -> Option<()> {
    let started = Instant::now();
    loop {
        let change = receiver.recv().await.ok()?;
        if tables_to_watch.contains(&change) {
            let elapsed = started.elapsed();

            if elapsed < min_interval {
                sleep(min_interval - elapsed).await;
            }

            break;
        }
    }

    // Drain the receiver
    while receiver.try_recv().is_ok() {}

    Some(())
}
