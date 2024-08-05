use super::{namespace::MessageNamespace, JsonRpcCall};
use crate::curve25519::{Curve25519PubKey, Curve25519SecKey};
use crate::network::swarm::SwarmAuth;
use crate::protos::Content;
use anyhow::bail;
use base64::prelude::*;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize)]
pub struct StoreMessageRequest<'a> {
    #[serde(rename = "pubKey")]
    recipient_pub_key: Cow<'a, str>,
    #[serde(rename = "data")]
    data_b64: String,
    ttl: usize,
    timestamp: u64,

    // Auth enabled parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    sig_timestamp: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "pubkey_ed25519")]
    pubkey_ed25519: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "signature")]
    signature_b64: Option<String>,
}

impl<'a> StoreMessageRequest<'a> {
    // pub fn new_authenticated<N: MessageNamespace>(
    //     auth: &'a impl SwarmAuth,
    //     recipient_pub_key: &'a Curve25519PubKey,
    //     data: &N::MessageType,
    //     ttl: usize,
    //     timestamp: u64,
    // ) -> anyhow::Result<Self> {
    //     if N::INT_VALUE == 0 {
    //         bail!("Namespace 0 is reserved for unauthenticated messages");
    //     }
    //
    //     let data = data.to_remote(auth)?;
    //     let verify_payload = format!("store{}{timestamp}", N::INT_VALUE);
    //     let signature = BASE64_STANDARD.encode(auth.sign(verify_payload.as_bytes()));
    //
    //     Ok(Self {
    //         recipient_pub_key: Cow::Borrowed(recipient_pub_key.hex()),
    //         data_b64: BASE64_STANDARD.encode(&data),
    //         ttl,
    //         timestamp,
    //         sig_timestamp: Some(timestamp),
    //         pubkey_ed25519: Some(auth.ed25519_pub_key().hex()),
    //         signature_b64: Some(signature),
    //     })
    // }

    pub fn new_unauthenticated(
        sender_key: &Curve25519SecKey,
        recipient_key: &Curve25519PubKey,
        content: &Content,
        ttl: usize,
        timestamp: u64,
    ) -> anyhow::Result<Self> {
        let content = content.encode_to_vec();

        todo!()
    }
}

#[derive(Debug, Deserialize)]
pub struct StoreMessageResponse {
    pub hash: String,
}

impl<'a> JsonRpcCall for StoreMessageRequest<'a> {
    type Response = StoreMessageResponse;

    fn method_name(&self) -> &'static str {
        "store"
    }

    fn create_response(&self, response: serde_json::Value) -> super::Result<Self::Response> {
        Ok(serde_json::from_value(response)?)
    }
}
