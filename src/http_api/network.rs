use crate::curve25519::Curve25519PubKey;
use crate::http_api::HttpApi;
use crate::network::Network;
use crate::utils::HttpBaseUrl;
use bytes::Bytes;
use http::StatusCode;

impl<N: Network> super::HttpCallSource for N {
    type Error = N::Error;
    type Arg<'a> = &'a Curve25519PubKey;

    async fn invoke<Api: HttpApi>(
        &self,
        base: &HttpBaseUrl,
        arg: Self::Arg<'_>,
        api: &Api,
    ) -> Result<Api::Response, Self::Error> {
        let resp = self
            .send_onion_proxied_request(
                base.as_ref(),
                &api.method(),
                api.request_content_type().as_ref().map(|s| s.as_ref()),
                api.request_body().as_ref().map(Bytes::as_ref),
                arg,
            )
            .await?;

        Ok(api.deserialize_response(StatusCode::OK, None, &resp))
    }
}
