use http::Method;
use url::Url;

use crate::oxenss::{Error as ApiError, JsonRpcCallSource};

impl JsonRpcCallSource for reqwest::Client {
    type Error = ApiError;
    type SourceArg<'a> = (Url, Method);

    async fn perform_raw_rpc(
        &self,
        (url, method): Self::SourceArg<'_>,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error> {
        Ok(self
            .request(method, url)
            .json(&req)
            .send()
            .await?
            .json()
            .await?)
    }
}
