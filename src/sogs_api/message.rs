use crate::clock::Timestamp;
use crate::session_id::{BlindedID, SessionID};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message<'a> {
    pub id: String,
    #[serde(rename = "session_id")]
    pub sender_id: Cow<'a, SessionID>,
    pub posted: Timestamp,
    pub edited: Option<Timestamp>,
    pub seqno: usize,
    pub whisper: bool,
    pub whisper_mods: bool,
    pub whisper_to: Option<Cow<'a, SessionID>>,
    pub data: Cow<'a, [u8]>,
    pub signature: Option<Cow<'a, str>>,
    pub reactions: HashMap<String, MessageReaction<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageReaction<'a> {
    pub index: isize,
    pub count: usize,
    pub reactors: Cow<'a, [Cow<'a, BlindedID>]>,
    pub you: bool,
}
