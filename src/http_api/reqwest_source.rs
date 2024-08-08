use super::HttpApi;
use crate::utils::HttpBaseUrl;
use http::header::{ACCEPT, CONTENT_TYPE, HOST};

impl super::HttpCallSource for reqwest::Client {
    type Error = reqwest::Error;
    type Arg<'a> = ();

    async fn invoke<Api: HttpApi>(
        &self,
        base: &HttpBaseUrl,
        _arg: Self::Arg<'_>,
        api: &Api,
    ) -> Result<Api::Response, Self::Error> {
        let url = api.full_url(base);
        let mut request = self.request(api.method(), url.clone());

        if let Some(host) = url.host_str() {
            request = request.header(HOST, host);
        }

        if let Some(content_type) = api.request_content_type() {
            request = request.header(CONTENT_TYPE, content_type.as_ref());
            if let Some(body) = api.request_body() {
                request = request.body(body);
            }
        }

        if let Some(accept) = api.expected_response_type() {
            request = request.header(ACCEPT, accept.as_ref());
        }

        let resp = request.send().await?;
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| Some(v.to_str().ok()?.to_string()));

        Ok(api.deserialize_response(
            resp.status(),
            content_type.as_ref().map(String::as_str),
            &resp.bytes().await?,
        ))
    }
}
