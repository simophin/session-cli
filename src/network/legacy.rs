use crate::curve25519::Curve25519PubKey;
use crate::oxenss::retrieve_service_node::{RetrieveServiceNode, ServiceNode};
use crate::oxenss::{Error as ApiError, JsonRpcCallSourceExt};
use crate::utils::NonEmpty;
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use derive_more::Display;
use http::Method;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use reqwest::Client;
use std::borrow::Cow;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::watch::Receiver;
use tokio::sync::{watch, Mutex};
use tokio::time::sleep;
use url::Url;

use super::{
    onion::{decrypt_onion_response, OnionRequestBuilder},
    Network, NetworkError, NetworkState, NodeAddress,
};

const PATH_EXPIRATION: Duration = Duration::from_secs(3600 * 24);

struct AvailableNetworkState {
    random_nodes: NonEmpty<ServiceNode>,
    path: NonEmpty<ServiceNode>,
    fetched_at: Instant,
}

struct ErrorNetworkState {
    what: LegacyNetworkError,
    when: Instant,
}

enum LegacyNetworkState {
    Init,
    Error(ErrorNetworkState),
    Available(Arc<AvailableNetworkState>),
}

pub struct LegacyNetwork {
    network_state_sender: watch::Sender<NetworkState>,
    state: Mutex<LegacyNetworkState>,
    seed_nodes: NonEmpty<Url>,
    client: Client,
}

#[derive(Error, Debug, Clone, Display)]
pub enum LegacyNetworkError {
    OnionEncryptionError(&'static str),
    JsonRpcError(#[from] Arc<ApiError>),
    NoUsableNodes,
    Timeout,
}

impl From<reqwest::Error> for LegacyNetworkError {
    fn from(e: reqwest::Error) -> Self {
        LegacyNetworkError::JsonRpcError(Arc::new(e.into()))
    }
}

impl From<ApiError> for LegacyNetworkError {
    fn from(e: ApiError) -> Self {
        LegacyNetworkError::JsonRpcError(Arc::new(e))
    }
}

impl NetworkError for LegacyNetworkError {
    fn should_retry(&self) -> bool {
        match self {
            LegacyNetworkError::JsonRpcError(e) => match e.as_ref() {
                ApiError::RequestError(e, _) => e.is_server_error(),
                _ => false,
            },
            LegacyNetworkError::OnionEncryptionError(_) => false,
            LegacyNetworkError::NoUsableNodes | LegacyNetworkError::Timeout => true,
        }
    }
}

impl LegacyNetwork {
    pub fn new(client: Client, seed_nodes: NonEmpty<Url>) -> Self {
        Self {
            network_state_sender: watch::channel(NetworkState::Idle).0,
            state: Mutex::new(LegacyNetworkState::Init),
            seed_nodes,
            client,
        }
    }

    async fn report_network_issue(&self, e: &LegacyNetworkError) {
        match e {
            LegacyNetworkError::Timeout => {}
            LegacyNetworkError::JsonRpcError(err) if matches!(err.as_ref(), ApiError::RequestError(code, _) if code.is_client_error()) =>
            {
                return;
            }
            LegacyNetworkError::NoUsableNodes => {}
            _ => return,
        }

        log::info!("Received error. Invalidate network state");
        let mut state = self.state.lock().await;
        *state = LegacyNetworkState::Error(ErrorNetworkState {
            what: e.clone(),
            when: Instant::now(),
        });

        let _ = self
            .network_state_sender
            .send(NetworkState::Error(LegacyNetworkError::Timeout.into()));
    }

    async fn perform_onion_req(
        &self,
        path_entry: &ServiceNode,
        dest_pub_key: &Curve25519PubKey,
        mut builder: OnionRequestBuilder,
        payload: &[u8],
    ) -> Result<Vec<u8>, <Self as Network>::Error> {
        let do_request = async move {
            let (payload, final_pub_key, final_sec_key) = builder
                .build(payload.as_ref())
                .map_err(LegacyNetworkError::OnionEncryptionError)?;

            let resp = self
                .client
                .post(path_entry.onion_req_url())
                .body(payload.as_ref().to_vec())
                .send()
                .await?;

            log::info!("Received response status={}", resp.status());

            let resp = resp.error_for_status()?;
            let body = resp.text().await?;

            let resp = decrypt_onion_response(
                &BASE64_STANDARD.decode(body).map_err(|e| {
                    ApiError::Other(anyhow!("Error decoding response as base64: {e:?}"))
                })?,
                dest_pub_key,
                &final_pub_key,
                &final_sec_key,
            )
            .ok_or(LegacyNetworkError::OnionEncryptionError(
                "Unable to decrypt onion response",
            ))?;

            Ok(resp.as_ref().to_vec())
        };

        let r = tokio::time::timeout(Duration::from_secs(10), do_request)
            .await
            .map_err(|_| LegacyNetworkError::Timeout)
            .and_then(|r| r);

        match &r {
            Ok(r) => log::debug!("Received {} bytes", r.len()),
            Err(e) => {
                log::error!("Error receiving onion http response: {e:?}");
                self.report_network_issue(e).await;
            }
        };

        r
    }

    async fn get_available_network_state(
        &self,
    ) -> Result<impl AsRef<AvailableNetworkState>, LegacyNetworkError> {
        let mut state = self.state.lock().await;
        match &*state {
            LegacyNetworkState::Error(ErrorNetworkState { what, when }) if what.should_retry() => {
                let delay = Duration::from_secs(5).saturating_sub(when.elapsed());
                log::info!("Delaying path update retry for {:?}ms", delay.as_millis());
                sleep(delay).await;
            }

            LegacyNetworkState::Error(ErrorNetworkState { what, .. }) => return Err(what.clone()),

            LegacyNetworkState::Available(state)
                if state.fetched_at.elapsed() < PATH_EXPIRATION =>
            {
                return Ok(state.clone());
            }

            _ => {}
        }

        let _ = self.network_state_sender.send(NetworkState::Connecting);

        let seed_node = self.seed_nodes.choose(&mut rand::thread_rng()).unwrap();
        log::info!("Fetching random nodes from: {seed_node}");

        async {
            let random_nodes: NonEmpty<ServiceNode> = self
                .client
                .perform_json_rpc(
                    (
                        seed_node.join("json_rpc").expect("To join json_rpc"),
                        Method::POST,
                    ),
                    &RetrieveServiceNode::new(25),
                )
                .await
                .map_err(Arc::new)?
                .try_into()
                .map_err(|_| LegacyNetworkError::NoUsableNodes)?;

            let path = NonEmpty::from_iter(
                random_nodes
                    .choose_multiple(&mut rand::thread_rng(), 3)
                    .cloned(),
            )
            .ok_or(LegacyNetworkError::NoUsableNodes)?;

            log::info!("Fetched {} random nodes", random_nodes.len());

            Ok(AvailableNetworkState {
                random_nodes,
                path,
                fetched_at: Instant::now(),
            })
        }
        .await
        .map(Arc::new)
        .inspect(|a| {
            *state = LegacyNetworkState::Available(a.clone());
            let _ = self
                .network_state_sender
                .send(NetworkState::Connected(a.path.map(|n| n.public_ip)));
        })
        .inspect_err(|e: &LegacyNetworkError| {
            *state = LegacyNetworkState::Error(ErrorNetworkState {
                what: e.clone(),
                when: Instant::now(),
            });
            let _ = self
                .network_state_sender
                .send(NetworkState::Error(e.clone().into()));
        })
    }
}

impl Network for LegacyNetwork {
    type Error = LegacyNetworkError;

    fn watch_state(&self) -> Receiver<NetworkState> {
        self.network_state_sender.subscribe()
    }

    async fn send_onion_request_to_node<'a>(
        &self,
        dest: NodeAddress<'a>,
        payload: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        let avail = self.get_available_network_state().await?;
        let AvailableNetworkState { path, .. } = avail.as_ref();
        let dest_pub_key = dest
            .x25519_pub_key
            .unwrap_or_else(|| Cow::Owned(dest.pub_key.to_curve25519()));

        let mut builder = OnionRequestBuilder::from_path(path.iter());
        builder.set_snode_destination(
            *dest.addr.ip(),
            dest.addr.port(),
            &dest.pub_key,
            dest_pub_key.as_ref(),
        );

        self.perform_onion_req(path.head(), dest_pub_key.as_ref(), builder, payload)
            .await
    }

    async fn send_onion_request_to_random_node(
        &self,
        payload: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        let avail = self.get_available_network_state().await?;
        let AvailableNetworkState { random_nodes, .. } = avail.as_ref();
        let node = random_nodes.choose_random(&mut thread_rng());

        self.send_onion_request_to_node(
            NodeAddress {
                addr: SocketAddrV4::new(*node.public_ip.as_ref(), node.storage_port),
                pub_key: Cow::Borrowed(&node.pubkey_ed25519),
                x25519_pub_key: Some(Cow::Borrowed(&node.pubkey_x25519)),
            },
            payload,
        )
        .await
    }

    async fn send_onion_proxied_request(
        &self,
        url: &Url,
        method: &Method,
        content_type: Option<&str>,
        body: Option<&[u8]>,
        dest_pub_key: &Curve25519PubKey,
    ) -> Result<Vec<u8>, Self::Error> {
        let avail = self.get_available_network_state().await?;
        let AvailableNetworkState { path, .. } = avail.as_ref();

        let mut builder = OnionRequestBuilder::from_path(path.iter());
        builder
            .set_server_destination(url, method, dest_pub_key)
            .map_err(|e| LegacyNetworkError::OnionEncryptionError(e))?;

        let payload = build_http_payload(
            url,
            method,
            content_type.map(Cow::Borrowed),
            body.map(Cow::Borrowed),
        );

        self.perform_onion_req(path.head(), dest_pub_key, builder, &payload)
            .await
    }
}

fn build_http_payload(
    url: &Url,
    method: &Method,
    content_type: Option<Cow<str>>,
    body: Option<Cow<[u8]>>,
) -> Vec<u8> {
    use std::io::Write;

    let mut payload = Vec::new();
    let path = url.path();
    if let Some(query) = url.query() {
        let _ = write!(payload, "{method} {path}?{query} HTTP/1.1\r\n");
    } else {
        let _ = write!(payload, "{method} {path} HTTP/1.1\r\n");
    }

    // Host header
    if let Some(host) = url.host_str() {
        let _ = write!(payload, "Host: {host}\r\n");
    }

    // If we have content type and body, add them
    if let (Some(content_type), Some(body)) = (content_type, body) {
        let _ = write!(payload, "Content-Type: {content_type}\r\n");
        let _ = write!(payload, "Content-Length: {}\r\n", body.len());
        let _ = write!(payload, "\r\n");
        payload.extend_from_slice(body.as_ref());
    } else {
        let _ = write!(payload, "\r\n");
    }

    payload
}

impl OnionRequestBuilder {
    pub fn from_path<'a>(path: impl Iterator<Item = &'a ServiceNode>) -> Self {
        let mut builder = Self::new();
        for node in path {
            builder.add_hop((&node.pubkey_x25519, &node.pubkey_ed25519));
        }
        builder
    }
}
