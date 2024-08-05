use crate::session_id::{BlindedID, GroupID, IndividualID, SessionID};
use derive_more::From;
use reqwest::Url;
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMode {
    Defaulted,
    All,
    Disabled,
    MentionsOnly,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExpiryMode {
    None,
    AfterSend,
    AfterRead,
}

#[derive(SerializeDisplay, DeserializeFromStr, Debug, From)]
pub enum MessageSource<'a> {
    IndividualSwarm(Cow<'a, IndividualID>),
    Blinded(Cow<'a, BlindedID>),
    GroupSwarm(Cow<'a, GroupID>),
    Community(Cow<'a, Url>),
}

impl ToSql for MessageSource<'_> {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.as_str().to_sql()
    }
}

impl Display for MessageSource<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<SessionID> for MessageSource<'static> {
    fn from(id: SessionID) -> Self {
        match id {
            SessionID::Individual(id) => MessageSource::IndividualSwarm(Cow::Owned(id)),
            SessionID::Group(id) => MessageSource::GroupSwarm(Cow::Owned(id)),
            SessionID::Blinded(id) => MessageSource::Blinded(Cow::Owned(id)),
        }
    }
}

impl FromStr for MessageSource<'static> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = IndividualID::from_str(s) {
            return Ok(MessageSource::IndividualSwarm(Cow::Owned(id)));
        }

        if let Ok(id) = GroupID::from_str(s) {
            return Ok(MessageSource::GroupSwarm(Cow::Owned(id)));
        }

        if let Ok(url) = Url::from_str(s) {
            return Ok(MessageSource::Community(Cow::Owned(url)));
        }

        anyhow::bail!("Invalid message source: {}", s)
    }
}

impl<'a> MessageSource<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            MessageSource::IndividualSwarm(id) => id.as_str(),
            MessageSource::GroupSwarm(id) => id.as_str(),
            MessageSource::Blinded(id) => id.as_str(),
            MessageSource::Community(url) => url.as_str(),
        }
    }
}
