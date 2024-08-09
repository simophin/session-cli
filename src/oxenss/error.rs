use anyhow::anyhow;
use derive_more::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
pub enum Error {
    #[display("HTTP request error: {} ({:?})", _0, _1)]
    RequestError(http::StatusCode, Option<String>),
    JsonParseError(#[from] serde_json::Error),
    Other(#[from] anyhow::Error),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        if let Some(status) = value.status() {
            return Error::RequestError(status, value.to_string().into());
        }

        Error::Other(anyhow!("Generic http error: {value:?}"))
    }
}
