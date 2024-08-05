use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::marker::PhantomData;

use super::namespace::MessageNamespace;
use super::{JsonRpcCall, StandardJsonRpcResponse};
use crate::clock::Timestamp;
use crate::network::swarm::SwarmAuth;
use crate::session_id::SessionID;
use crate::{base64::Base64, ed25519::ED25519PubKey};

#[derive(Debug, Serialize)]
pub struct RetrieveMessageRequest<'a, N> {
    #[serde(rename = "pubkey")]
    session_id: String,
    last_hash: &'a str,
    pubkey_ed25519: Option<Cow<'a, ED25519PubKey>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_size: Option<usize>,
    #[serde(flatten)]
    signature: Value,
    timestamp: Timestamp,
    namespace: isize,
    #[serde(skip)]
    phantom: PhantomData<N>,
}

impl<'a, NS: MessageNamespace> JsonRpcCall for RetrieveMessageRequest<'a, NS> {
    type Response = RetrieveMessageResponse;

    fn method_name(&self) -> &'static str {
        "retrieve"
    }

    fn namespace(&self) -> Option<isize> {
        Some(NS::INT_VALUE)
    }

    fn create_response(&self, response: Value) -> super::Result<Self::Response> {
        StandardJsonRpcResponse::body_from_value(response)
    }
}

impl<'a, NS: MessageNamespace> RetrieveMessageRequest<'a, NS> {
    pub fn new(
        auth: &'a impl SwarmAuth,
        last_hash: Option<&'a str>,
        max_size: Option<usize>,
        timestamp: Timestamp,
    ) -> anyhow::Result<Self> {
        let namespace = NS::INT_VALUE;

        let sig_payload = if namespace == 0 {
            format!("retrieve{timestamp}")
        } else {
            format!("retrieve{namespace}{timestamp}")
        };

        let signature = serde_json::to_value(
            auth.sign(sig_payload.as_bytes())
                .context("Signing is required")?,
        )?;

        let session_id: SessionID = auth.session_id().into_owned().into();

        Ok(Self {
            session_id: session_id.to_string(),
            last_hash: last_hash.unwrap_or_default(),
            pubkey_ed25519: match session_id {
                SessionID::Individual(_) => Some(auth.ed25519_pub_key()),
                _ => None,
            },
            max_size,
            signature,
            timestamp,
            namespace,
            phantom: PhantomData,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RetrieveMessageResponse {
    pub messages: Vec<Message>,
    pub more: bool,
    #[serde(rename = "t")]
    pub timestamp: Timestamp,
}

impl RetrieveMessageResponse {
    pub fn latest_hash(&self) -> Option<&str> {
        self.messages
            .iter()
            .max_by_key(|item| item.created)
            .map(|s| s.hash.as_str())
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub data: Base64<Vec<u8>>,
    pub hash: String,
    pub expiration: Timestamp,
    #[serde(rename = "timestamp")]
    pub created: Timestamp,
}
