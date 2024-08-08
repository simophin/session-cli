use super::{Error as ApiError, JsonRpcCall, JsonRpcCallSource, Result};
use crate::curve25519::Curve25519PubKey;
use crate::ed25519::ED25519PubKey;
use crate::ip::PublicIPv4;
use crate::network::NodeAddress;
use http::Method;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddrV4;
use url::Url;

#[derive(Serialize)]
pub struct RetrieveServiceNode {
    active_only: bool,
    limit: usize,
    fields: HashMap<&'static str, bool>,
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct ServiceNode {
    pub public_ip: PublicIPv4,
    pub storage_port: u16,
    pub pubkey_x25519: Curve25519PubKey,
    pub pubkey_ed25519: ED25519PubKey,
}

impl ServiceNode {
    pub fn storage_rpc_url(&self) -> Url {
        Url::parse(&format!(
            "https://{}:{}/storage_rpc/v1",
            self.public_ip, self.storage_port
        ))
        .unwrap()
    }

    pub fn onion_req_url(&self) -> Url {
        Url::parse(&format!(
            "https://{}:{}/onion_req/v2",
            self.public_ip, self.storage_port
        ))
        .unwrap()
    }

    pub fn address(&self) -> NodeAddress {
        return NodeAddress {
            addr: SocketAddrV4::new(*self.public_ip.as_ref(), self.storage_port),
            pub_key: Cow::Borrowed(&self.pubkey_ed25519),
            x25519_pub_key: Some(Cow::Borrowed(&self.pubkey_x25519)),
        };
    }
}

impl JsonRpcCallSource for ServiceNode {
    type Error = ApiError;
    type SourceArg<'a> = &'a Client;

    async fn perform_raw_rpc(&self, arg: Self::SourceArg<'_>, req: Value) -> Result<Value> {
        arg.perform_raw_rpc((self.storage_rpc_url(), Method::POST), req)
            .await
    }
}

#[derive(Deserialize)]
struct RetrieveServiceNodeResponse {
    result: RetrieveServiceNodeResponseResult,
}

#[derive(Deserialize)]
struct RetrieveServiceNodeResponseResult {
    service_node_states: Vec<ServiceNode>,
}

impl JsonRpcCall for RetrieveServiceNode {
    type Response = Vec<ServiceNode>;

    fn method_name(&self) -> &'static str {
        "get_n_service_nodes"
    }

    fn create_response(&self, response: Value) -> Result<Self::Response> {
        Ok(
            serde_json::from_value::<RetrieveServiceNodeResponse>(response)?
                .result
                .service_node_states,
        )
    }
}

impl RetrieveServiceNode {
    pub fn new(limit: usize) -> Self {
        Self {
            active_only: true,
            limit,
            fields: [
                "public_ip",
                "storage_port",
                "pubkey_x25519",
                "pubkey_ed25519",
            ]
            .into_iter()
            .map(|name| (name, true))
            .collect(),
        }
    }
}
