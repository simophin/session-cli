use crate::message_crypto::strip_message_padding;
use crate::network::swarm::SwarmAuth;
use crate::oxenss::namespace::{DefaultNamespace, GroupNamespace};
use crate::protos::{Content, Envelope, WebSocketMessage, WebSocketRequestMessage};
use crate::session_id::IndividualOrBlindedID;
use anyhow::{bail, Context};
use prost::Message;

#[derive(Clone, Debug)]
pub struct RegularMessage {
    pub sender: IndividualOrBlindedID,
    pub content: Content,
}

pub trait RegularMessageDecoder: Sized {
    fn decode_and_decrypt(
        input: &[u8],
        swarm_auth: &impl SwarmAuth,
    ) -> anyhow::Result<RegularMessage>;
}

impl RegularMessageDecoder for GroupNamespace {
    fn decode_and_decrypt(
        input: &[u8],
        swarm_auth: &impl SwarmAuth,
    ) -> anyhow::Result<RegularMessage> {
        let (session_id, content) = swarm_auth.decrypt(input)?;

        let Envelope {
            content: Some(content),
            ..
        } = Envelope::decode(content.as_ref()).context("Decode envelope")?
        else {
            bail!("No content in envelope");
        };

        let content = Content::decode(content.as_slice()).context("Decode content")?;
        Ok(RegularMessage {
            sender: session_id.into(),
            content,
        })
    }
}

impl RegularMessageDecoder for DefaultNamespace {
    fn decode_and_decrypt(
        input: &[u8],
        swarm_auth: &impl SwarmAuth,
    ) -> anyhow::Result<RegularMessage> {
        let WebSocketMessage {
            request:
                Some(WebSocketRequestMessage {
                    body: Some(body), ..
                }),
            ..
        } = WebSocketMessage::decode(input).context("Decode websocket")?
        else {
            bail!("No body in websocket message");
        };

        let Envelope {
            content: Some(content),
            ..
        } = Envelope::decode(body.as_slice()).context("Decode envelope")?
        else {
            bail!("No content in envelope");
        };

        let (session_id, content) = swarm_auth.decrypt(&content)?;
        let content =
            Content::decode(strip_message_padding(content.as_ref())).context("Decode content")?;

        Ok(RegularMessage {
            sender: session_id.into(),
            content,
        })
    }
}
