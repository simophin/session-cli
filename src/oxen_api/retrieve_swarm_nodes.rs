use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::net::SocketAddrV4;

use crate::curve25519::Curve25519PubKey;
use crate::ed25519::ED25519PubKey;
use crate::ip::PublicIPv4;
use crate::network::NodeAddress;
use crate::session_id::IndividualOrGroupID;

use super::{JsonRpcCall, Result};

#[derive(Serialize)]
pub struct RetrieveSwarmNodes<'a> {
    #[serde(rename = "pubKey")]
    pub session_id: &'a IndividualOrGroupID,
}

#[derive(Deserialize, Debug)]
pub struct SwarmNode {
    pub ip: PublicIPv4,
    #[serde(rename = "port_https")]
    pub port: u16,
    pub pubkey_ed25519: ED25519PubKey,
    pub pubkey_x25519: Curve25519PubKey,
}

impl SwarmNode {
    pub fn address(&self) -> NodeAddress {
        NodeAddress {
            addr: SocketAddrV4::new(*self.ip.as_ref(), self.port),
            pub_key: Cow::Borrowed(&self.pubkey_ed25519),
            x25519_pub_key: Some(Cow::Borrowed(&self.pubkey_x25519)),
        }
    }
}

impl<'a> JsonRpcCall for RetrieveSwarmNodes<'a> {
    type Response = Vec<SwarmNode>;

    fn method_name(&self) -> &'static str {
        "get_snodes_for_pubkey"
    }

    fn create_response(&self, response: Value) -> Result<Self::Response> {
        #[derive(Deserialize, Debug)]
        struct ResponseBody {
            snodes: Vec<SwarmNode>,
        }

        #[derive(Deserialize, Debug)]
        struct Response {
            body: String,
        }

        let Response { body } = serde_json::from_value(response)?;
        let ResponseBody { snodes } = serde_json::from_str(&body)?;

        Ok(snodes)
    }
}
