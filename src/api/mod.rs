mod error;
pub mod seed;
pub(crate) mod service_node;

pub use error::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;

type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Debug)]
struct JsonRpcRequest<'a, T> {
    method: &'a str,
    params: &'a T,
}

async fn json_rpc<Params: Serialize, T: DeserializeOwned>(
    url: &str,
    method: &str,
    params: &Params,
) -> Result<T> {
    let value = serde_json::to_value(&JsonRpcRequest { method, params })?;
    log::debug!("Sending JSON-RPC request to {url}: {value}");

    let resp = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .post(url)
        .json(&value)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    log::debug!("Received JSON-RPC response: {resp}");
    Ok(serde_json::from_value(resp)?)
}
