use anyhow::{anyhow, Context};
use http::StatusCode;
use serde::{Deserialize, Serialize};

pub trait JsonRpcCall: Serialize {
    type Response;

    fn method_name(&self) -> &'static str;
    fn namespace(&self) -> Option<isize> {
        None
    }

    fn create_response(&self, response: serde_json::Value) -> super::Result<Self::Response>;
}

pub trait JsonRpcCallSource {
    type Error: std::error::Error + Send + Sync + From<super::Error> + 'static;
    type SourceArg<'a>;

    async fn perform_raw_rpc(
        &self,
        arg: Self::SourceArg<'_>,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error>;
}

pub trait JsonRpcCallSourceExt: JsonRpcCallSource {
    async fn perform_json_rpc<R: JsonRpcCall>(
        &self,
        arg: Self::SourceArg<'_>,
        req: &R,
    ) -> Result<R::Response, Self::Error>;
}

impl<T: JsonRpcCallSource> JsonRpcCallSourceExt for T {
    async fn perform_json_rpc<R: JsonRpcCall>(
        &self,
        arg: Self::SourceArg<'_>,
        req: &R,
    ) -> Result<R::Response, Self::Error> {
        let req_body =
            serde_json::to_value(&JsonRpcRequest::from_rpc(req)).map_err(super::Error::from)?;
        log::debug!("Sending RPC request: {req_body:#?}");
        let resp = self.perform_raw_rpc(arg, req_body).await?;
        log::debug!("Received RPC response: {:#?}", resp);

        Ok(req.create_response(resp)?)
    }
}

#[derive(Serialize, Debug)]
struct JsonRpcRequest<'a, T> {
    pub method: &'a str,
    pub params: &'a T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<isize>,
}

impl<'a, T> JsonRpcRequest<'a, T> {
    pub fn from_rpc(rpc: &'a T) -> Self
    where
        T: JsonRpcCall,
    {
        Self {
            method: rpc.method_name(),
            params: rpc,
            namespace: rpc.namespace(),
        }
    }
}

#[derive(Debug)]
pub struct StandardJsonRpcResponse<B>(pub B);

impl<B> StandardJsonRpcResponse<B>
where
    B: for<'d> Deserialize<'d>,
{
    pub fn body_from_value(value: serde_json::Value) -> super::Result<B> {
        match value
            .get("status")
            .or_else(|| value.get("code"))
            .context("Expecting a status/code field")
            .and_then(|v| {
                v.as_i64()
                    .context("Expecting status/code field to be an integer")
            })
            .and_then(|v| u16::try_from(v).context("Expecting status/code to be a small number"))
            .and_then(|v| StatusCode::try_from(v).context("Expecting a valid http status code"))
        {
            Ok(status_code) if status_code.is_success() => {
                let body = value
                    .get("body")
                    .ok_or_else(|| anyhow!("Expecting a body field"))?;
                let value = serde_json::from_value(body.clone())?;
                Ok(value)
            }

            Ok(status_code) => {
                let body = value
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                return Err(super::Error::RequestError(
                    status_code,
                    Some(body.to_string()),
                ));
            }

            Err(e) => return Err(super::Error::Other(e)),
        }
    }
}
