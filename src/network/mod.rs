pub mod batch;
pub mod legacy;
mod onion;
pub mod swarm;

use crate::curve25519::Curve25519PubKey;
use crate::ed25519::ED25519PubKey;
use crate::ip::PublicIPv4;
use crate::oxen_api::{Error as ApiError, JsonRpcCallSource};
use crate::utils::NonEmpty;
use derive_more::Display;
use http::Request;
use std::borrow::Cow;
use std::fmt::Debug;
use std::net::SocketAddrV4;
use thiserror::Error;
use tokio::sync::watch;

pub trait NetworkError: std::error::Error + Send + Sync + 'static {
    fn should_retry(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAddress<'a> {
    pub addr: SocketAddrV4,
    pub pub_key: Cow<'a, ED25519PubKey>,
    pub x25519_pub_key: Option<Cow<'a, Curve25519PubKey>>,
}

impl<'a> NodeAddress<'a> {
    pub fn into_owned(self) -> NodeAddress<'static> {
        NodeAddress {
            addr: self.addr,
            pub_key: Cow::Owned(self.pub_key.into_owned()),
            x25519_pub_key: self.x25519_pub_key.map(|k| Cow::Owned(k.into_owned())),
        }
    }
}

pub enum OnionRequest<'a> {
    Node {
        dest: Option<NodeAddress<'a>>,
        payload: Cow<'a, [u8]>,
    },
    Http {
        request: Request<Cow<'a, [u8]>>,
        dest_key: &'a Curve25519PubKey,
    },
}

pub enum NetworkState {
    Idle,
    Connecting,
    Connected(NonEmpty<PublicIPv4>),
    Error(anyhow::Error),
}

pub trait Network: 'static {
    type Error: NetworkError;

    async fn send_onion_request(&self, req: OnionRequest<'_>) -> Result<Vec<u8>, Self::Error>;

    fn watch_state(&self) -> watch::Receiver<NetworkState>;
}

#[derive(Error, Debug, Display)]
pub enum JsonRpcNetworkError<NE> {
    NetworkError(NE),
    JsonRpcError(#[from] ApiError),
}

impl<N: Network> JsonRpcCallSource for N {
    type Error = JsonRpcNetworkError<N::Error>;
    type SourceArg<'s> = Option<NodeAddress<'s>>;

    async fn perform_raw_rpc(
        &self,
        arg: Self::SourceArg<'_>,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error> {
        let payload = serde_json::to_vec(&req).map_err(ApiError::from)?;
        log::debug!(
            "Sending onion request to {arg:?}: {}",
            std::str::from_utf8(&payload).unwrap()
        );
        self.send_onion_request(OnionRequest::Node {
            dest: arg,
            payload: Cow::Owned(payload),
        })
        .await
        .map_err(JsonRpcNetworkError::NetworkError)
        .and_then(|data| {
            serde_json::from_slice(&data).map_err(|e| JsonRpcNetworkError::JsonRpcError(e.into()))
        })
        .inspect(|r| {
            log::debug!("Received onion response: {r:#?}");
        })
        .inspect_err(|e| {
            log::error!("Error sending onion request: {e:?}");
        })
    }
}
