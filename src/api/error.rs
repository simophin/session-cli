use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while posting request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Error while deserializing response: {0}")]
    JsonParseError(#[from] serde_json::Error),
}