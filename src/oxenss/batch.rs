use super::{JsonRpcCall, StandardJsonRpcResponse};
use crate::utils::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
pub struct BatchRequest<'a> {
    pub requests: &'a [&'a Value],
}

#[derive(Deserialize, Debug)]
pub struct BatchResponse {
    pub results: Vec<Value>,
}

impl<'a> JsonRpcCall for BatchRequest<'a> {
    type Response = BatchResponse;

    fn method_name(&self) -> &'static str {
        "batch"
    }

    fn create_response(&self, response: Value) -> super::Result<Self::Response> {
        let r: Json<BatchResponse> = StandardJsonRpcResponse::body_from_value(response)?;
        Ok(r.0)
    }
}
