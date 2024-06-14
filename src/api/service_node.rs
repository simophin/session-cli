use std::fmt::Display;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::hex_encode::Hex;

#[derive(Serialize)]
struct GetSwarmRequestParams {
    #[serde(rename = "pubKey")]
    pub pub_key: String,
}

#[derive(Deserialize)]
struct GetSwarmResponse {
    pub snodes: Vec<ServiceNode>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServiceNode {
    pub public_ip: IpAddr,
    pub storage_port: u16,
    pub pubkey_x25519: Hex<32>,
    pub pubkey_ed25519: Hex<32>,
}

impl ServiceNode {
    pub fn storage_url(&self) -> String {
        format!(
            "https://{}:{}/storage_rpc/v1",
            self.public_ip, self.storage_port
        )
    }
}

pub async fn get_swarm_nodes(
    service_node: &ServiceNode,
    pub_key: &str,
) -> super::Result<Vec<ServiceNode>> {
    return super::json_rpc(
        &service_node.storage_url(),
        "get_snodes_for_pubkey",
        &GetSwarmRequestParams {
            pub_key: pub_key.to_string(),
        },
    )
    .await
    .map(|response: GetSwarmResponse| response.snodes);
}
