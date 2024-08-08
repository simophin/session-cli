use bytes::Bytes;
use http::Method;
use std::borrow::Cow;
use url::Url;

pub use call_source::*;
pub use json::*;

use crate::utils::HttpBaseUrl;

mod build;
mod call_source;
mod json;
mod network;
mod reqwest_source;

pub trait HttpApi {
    type Response;

    fn method(&self) -> Method;
    fn path_segments(&self) -> impl Iterator<Item = Cow<str>>;
    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)>;

    fn request_content_type(&self) -> Option<Cow<str>>;

    fn request_body(&self) -> Option<Bytes>;

    fn expected_response_type(&self) -> Option<Cow<str>>;

    fn deserialize_response(
        &self,
        status_code: http::StatusCode,
        content_type: Option<&str>,
        buf: &[u8],
    ) -> Self::Response;

    fn full_url(&self, base: &HttpBaseUrl) -> Url {
        let builder = self
            .path_segments()
            .fold(base.build_upon(), |builder, segment| {
                builder.append_path(segment.as_ref())
            });

        self.queries()
            .fold(builder, |builder, (name, value)| {
                builder.append_query(name.as_ref(), value.as_ref())
            })
            .build()
    }
}
