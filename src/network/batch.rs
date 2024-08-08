use crate::oxenss::batch::BatchResponse;
use crate::oxenss::{
    batch::BatchRequest as ApiBatchRequest, Error as ApiError, JsonRpcCallSource,
    JsonRpcCallSourceExt,
};
use derive_more::Display;
use futures_util::future::select_all;
use std::collections::VecDeque;
use std::future::pending;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep_until;

pub struct BatchManager<'a, S: JsonRpcCallSource> {
    source: &'a S,
    batch_rpc_request: mpsc::Sender<(S::SourceArg<'static>, BatchRequest<S>)>,
}

struct BatchRequest<S: JsonRpcCallSource> {
    pub request_body: serde_json::Value,
    pub result_sender: oneshot::Sender<Result<serde_json::Value, BatchError<S::Error>>>,
}

struct BatchState<S: JsonRpcCallSource> {
    arg: S::SourceArg<'static>,
    batched: Vec<BatchRequest<S>>,
    deadline: Instant,
}

pub trait BatchKey {
    fn key(&self) -> impl Eq;
}

pub struct BatchManagerRunner<S: JsonRpcCallSource> {
    rx: mpsc::Receiver<(S::SourceArg<'static>, BatchRequest<S>)>,
}

const BATCH_MAX_WINDOW_DURATION: Duration = Duration::from_millis(100);

impl<'a, S> BatchManager<'a, S>
where
    S: JsonRpcCallSource,
    for<'s> <S as JsonRpcCallSource>::SourceArg<'s>: BatchKey,
{
    pub fn new(source: &'a S) -> (Self, BatchManagerRunner<S>) {
        let (tx, rx) = mpsc::channel(25);
        (
            Self {
                source,
                batch_rpc_request: tx,
            },
            BatchManagerRunner { rx },
        )
    }

    pub async fn run(
        &self,
        BatchManagerRunner { mut rx }: BatchManagerRunner<S>,
    ) -> anyhow::Result<()> {
        let mut states: VecDeque<BatchState<S>> = VecDeque::new();
        let mut tasks = Vec::new();

        loop {
            let delay_until_next_batch = async {
                match states.front() {
                    Some(front) => {
                        sleep_until(front.deadline.into()).await;
                        states.pop_front().unwrap()
                    }
                    None => pending().await,
                }
            };

            let drive_tasks = async {
                if tasks.is_empty() {
                    pending::<()>().await;
                    Ok(())
                } else {
                    let (r, index, _) = select_all(tasks.iter_mut()).await;
                    let _ = tasks.remove(index);
                    r
                }
            };

            let r = select! {
                state = delay_until_next_batch => {
                    tasks.push(Box::pin(self.handle_swarm_batch(state)));
                    continue;
                }
                r2 = drive_tasks => {
                    if let Err(e) = r2 {
                        log::error!("Failed to handle batch: {:?}", e);
                    }
                    continue;
                }
                r3 = rx.recv() => r3,
            };

            if let Some((arg, req)) = r {
                match states.iter_mut().find(|s| s.arg.key() == arg.key()) {
                    Some(state) => state.batched.push(req),
                    None => states.push_back(BatchState {
                        arg,
                        batched: vec![req],
                        deadline: Instant::now() + BATCH_MAX_WINDOW_DURATION,
                    }),
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    async fn handle_swarm_batch(&self, state: BatchState<S>) -> Result<(), S::Error> {
        let BatchState { arg, batched, .. } = state;

        let results = self
            .source
            .perform_json_rpc(
                arg,
                &ApiBatchRequest {
                    requests: batched
                        .iter()
                        .map(|b| &b.request_body)
                        .collect::<Vec<_>>()
                        .as_slice(),
                },
            )
            .await;

        match results {
            Ok(BatchResponse { results }) => {
                for (req, result) in batched.into_iter().zip(results.into_iter()) {
                    let _ = req.result_sender.send(Ok(result));
                }
            }
            Err(e) => {
                let e = Arc::new(e);
                for req in batched {
                    let _ = req
                        .result_sender
                        .send(Err(BatchError::SourceError(e.clone())));
                }
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug, Display)]
pub enum BatchError<E> {
    #[display(fmt = "Source error: {}", _0)]
    SourceError(Arc<E>),

    #[display(fmt = "Batch error: {}", _0)]
    BatchError(anyhow::Error),

    Cancelled,
}

impl<E: From<ApiError>> From<ApiError> for BatchError<E> {
    fn from(e: ApiError) -> Self {
        BatchError::SourceError(Arc::new(e.into()))
    }
}

impl<'a, S> JsonRpcCallSource for BatchManager<'a, S>
where
    S: JsonRpcCallSource,
    for<'s> <S as JsonRpcCallSource>::SourceArg<'s>: BatchKey + 'static,
{
    type Error = BatchError<S::Error>;
    type SourceArg<'s> = S::SourceArg<'static>;

    async fn perform_raw_rpc(
        &self,
        arg: Self::SourceArg<'_>,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error> {
        let (tx, rx) = oneshot::channel();
        let req = BatchRequest {
            request_body: req,
            result_sender: tx,
        };

        log::debug!("Sending batched request: {:#?}", req.request_body);
        self.batch_rpc_request
            .send((arg, req))
            .await
            .map_err(|_| BatchError::Cancelled)?;

        rx.await
            .map_err(|_| BatchError::Cancelled)
            .inspect(|result| {
                log::debug!("Received batched response: {:?}", result);
            })?
    }
}
