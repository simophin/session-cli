use anyhow::Context;
use bytes::BufMut;
use http::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use std::io::Write;

mod build;
mod call_source;
mod json;
mod legacy;
mod reqwest_source;

pub use build::*;
pub use call_source::*;
pub use json::*;

pub trait HttpApi {
    type Response;

    fn method(&self) -> Method;
    fn path_segments(&self) -> impl Iterator<Item = Cow<str>>;
    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)>;

    fn request_content_type(&self) -> Option<Cow<str>>;

    fn write_request_body(&self, buf: impl BufMut) -> anyhow::Result<()>;

    fn expected_response_type(&self) -> Option<Cow<str>>;

    fn deserialize_response(
        &self,
        status_code: http::StatusCode,
        content_type: Option<&str>,
        buf: &[u8],
    ) -> Self::Response;
}
