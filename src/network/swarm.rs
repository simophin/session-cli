use super::{JsonRpcNetworkError, Network, NetworkError, NodeAddress};
use crate::ed25519::ED25519PubKey;
use crate::oxenss::retrieve_swarm_nodes::RetrieveSwarmNodes;
use crate::oxenss::{Error as ApiError, JsonRpcCallSource, JsonRpcCallSourceExt};
use crate::session_id::{IndividualID, IndividualOrGroupID, SessionID};
use crate::utils::NonEmpty;
use derive_more::Display;
use rand::prelude::SliceRandom;
use serde::Serialize;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct SwarmManager<'a, N> {
    network: &'a N,
}

#[derive(Error, Display, Debug, Clone)]
pub enum SwarmRequestError {
    #[display("No usable nodes")]
    JsonError(Arc<serde_json::Error>),

    #[display("Network error: {}", _0)]
    OnionNetworkError(Arc<dyn NetworkError>),

    #[display("Json RPC error: {}", _0)]
    JsonRpcError(Arc<ApiError>),

    NoUsableNodes,
}

impl SwarmRequestError {
    pub fn should_retry(&self) -> bool {
        match self {
            SwarmRequestError::JsonError(_) | SwarmRequestError::JsonRpcError(_) => false,
            SwarmRequestError::OnionNetworkError(e) => e.should_retry(),
            SwarmRequestError::NoUsableNodes => true,
        }
    }
}

impl From<ApiError> for SwarmRequestError {
    fn from(e: ApiError) -> Self {
        SwarmRequestError::JsonRpcError(Arc::new(e))
    }
}

enum SwarmStateInner {
    Init,
    Error {
        error: SwarmRequestError,
        error_at: Instant,
    },
    Ready {
        usable: NonEmpty<NodeAddress<'static>>,
    },
}

#[derive(Clone)]
pub struct SwarmState(IndividualOrGroupID, Arc<Mutex<SwarmStateInner>>);

impl SwarmState {
    pub fn new(session_id: IndividualOrGroupID) -> Self {
        Self(session_id, Arc::new(Mutex::new(SwarmStateInner::Init)))
    }
}

impl super::batch::BatchKey for SwarmState {
    fn key(&self) -> impl Eq {
        &self.0
    }
}

pub trait SwarmAuth {
    type SessionIDType: Clone + Sized + Into<SessionID> + Display;

    fn sign(&self, payload: &[u8]) -> Option<impl Serialize + 'static>;
    fn decrypt(&self, payload: &[u8])
        -> anyhow::Result<(IndividualID, impl AsRef<[u8]> + 'static)>;
    fn session_id(&self) -> Cow<Self::SessionIDType>;
    fn ed25519_pub_key(&self) -> Cow<ED25519PubKey>;
}

impl<'a, N> SwarmManager<'a, N>
where
    N: Network,
{
    pub fn new(network: &'a N) -> Self {
        Self { network }
    }

    async fn retrieve_random_swarm_nodes(
        &self,
        state: &SwarmState,
    ) -> Result<NonEmpty<NodeAddress<'static>>, SwarmRequestError> {
        let mut inner = state.1.lock().await;
        match &*inner {
            SwarmStateInner::Error { error, .. } if !error.should_retry() => {
                return Err(error.clone());
            }

            SwarmStateInner::Error { error_at, .. } => {
                let delay = Duration::from_secs(5).saturating_sub(error_at.elapsed());
                log::info!("Retrying swarm request in {:?}ms", delay.as_millis());
                sleep(delay).await;
            }

            SwarmStateInner::Ready { usable } => {
                return Ok(usable.clone());
            }

            SwarmStateInner::Init => {}
        };

        self.network
            .perform_json_rpc(
                None,
                &RetrieveSwarmNodes {
                    session_id: &state.0.clone().into(),
                },
            )
            .await
            .map_err(|e| match e {
                JsonRpcNetworkError::NetworkError(e) => {
                    SwarmRequestError::OnionNetworkError(Arc::new(e))
                }
                JsonRpcNetworkError::JsonRpcError(e) => {
                    SwarmRequestError::JsonRpcError(Arc::new(e))
                }
            })
            .and_then(|nodes| {
                NonEmpty::from_iter(nodes.into_iter().map(|n| n.address().into_owned()))
                    .ok_or(SwarmRequestError::NoUsableNodes)
            })
            .inspect(|nodes| {
                *inner = SwarmStateInner::Ready {
                    usable: nodes.clone(),
                };
            })
            .inspect_err(|e| {
                *inner = SwarmStateInner::Error {
                    error: e.clone(),
                    error_at: Instant::now(),
                };
            })
    }

    pub async fn send_onion_request_to_swarm(
        &self,
        state: &SwarmState,
        payload: &[u8],
    ) -> Result<Vec<u8>, SwarmRequestError> {
        let mut nodes = self.retrieve_random_swarm_nodes(state).await?.into_inner();

        while let Some(addr) = nodes.choose(&mut rand::thread_rng()) {
            match self
                .network
                .send_onion_request_to_node(addr.clone(), payload)
                .await
            {
                Ok(resp) => return Ok(resp),
                Err(e) if e.should_retry() => {
                    log::warn!("Error sending request to {addr:?}: {e:?}. Retrying");
                    if let Some(index) = nodes.iter().position(|n| n == addr) {
                        nodes.remove(index);
                    }
                    continue;
                }
                Err(e) => return Err(SwarmRequestError::OnionNetworkError(Arc::new(e))),
            }
        }

        Err(SwarmRequestError::NoUsableNodes)
    }
}

impl<'a, N> JsonRpcCallSource for SwarmManager<'a, N>
where
    N: Network,
{
    type Error = SwarmRequestError;
    type SourceArg<'s> = SwarmState;

    async fn perform_raw_rpc(
        &self,
        arg: Self::SourceArg<'_>,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error> {
        log::debug!("Sending request to {}: {req:#?}", arg.0);
        let req =
            serde_json::to_vec(&req).map_err(|e| SwarmRequestError::JsonError(Arc::new(e)))?;
        self.send_onion_request_to_swarm(&arg, &req)
            .await
            .and_then(|resp| {
                serde_json::from_slice(&resp).map_err(|e| SwarmRequestError::JsonError(Arc::new(e)))
            })
            .inspect(|e| {
                log::debug!("Received response from {}: {e:#?}", arg.0);
            })
            .inspect_err(|e| {
                log::error!("Error sending request to {}: {e:?}", arg.0);
            })
    }
}
