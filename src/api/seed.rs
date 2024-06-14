use serde::{Deserialize, Serialize};

use crate::api::service_node::ServiceNode;
use crate::rand_util::RandExt;

#[derive(Serialize)]
pub struct GetServiceNodeParams {
    pub active_only: bool,
    pub limit: usize,
    pub fields: GetServiceNodeFields,
}

#[derive(Serialize)]
pub struct GetServiceNodeFields {
    pub public_ip: bool,
    pub storage_port: bool,
    pub pubkey_x25519: bool,
    pub pubkey_ed25519: bool,
}

#[derive(Deserialize)]
struct SeedResponse<T> {
    result: T,
}

const SEED_NODES: &[&str] = &[
    "https://seed1.getsession.org/json_rpc",
    "https://seed2.getsession.org/json_rpc",
];
pub async fn get_service_nodes(limit: usize) -> super::Result<Vec<ServiceNode>> {
    #[derive(Deserialize)]
    struct ServiceNodeResponse {
        service_node_states: Vec<ServiceNode>,
    }

    let params = GetServiceNodeParams {
        active_only: true,
        limit,
        fields: GetServiceNodeFields {
            public_ip: true,
            storage_port: true,
            pubkey_x25519: true,
            pubkey_ed25519: true,
        },
    };

    return super::json_rpc(SEED_NODES.rand_ref(), "get_n_service_nodes", &params)
        .await
        .map(|s: SeedResponse<ServiceNodeResponse>| s.result.service_node_states);
}
