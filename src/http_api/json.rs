use crate::http_api::HttpApi;
use anyhow::Context;
use bytes::BufMut;
use derive_more::Display;
use http::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use thiserror::Error;

pub trait HttpJsonApi {
    type SuccessResponse: DeserializeOwned;

    fn method(&self) -> Method;
    fn path_segments(&self) -> impl Iterator<Item = Cow<str>>;
    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> + '_;

    fn request(&self) -> Option<&impl Serialize>;
}

impl<T: HttpJsonApi> HttpApi for T {
    type Response = Result<<Self as HttpJsonApi>::SuccessResponse, HttpJsonApiError>;

    fn method(&self) -> Method {
        <Self as HttpJsonApi>::method(self)
    }

    fn path_segments(&self) -> impl Iterator<Item = Cow<str>> {
        <Self as HttpJsonApi>::path_segments(self)
    }

    fn queries(&self) -> impl Iterator<Item = (Cow<str>, Cow<str>)> {
        <Self as HttpJsonApi>::queries(self)
    }

    fn request_content_type(&self) -> Option<Cow<str>> {
        Some(Cow::Borrowed("application/json"))
    }

    fn write_request_body(&self, buf: impl BufMut) -> anyhow::Result<()> {
        if let Some(body) = self.request() {
            serde_json::to_writer(buf.writer(), body).context("Serializing request to JSON")?;
        }

        Ok(())
    }

    fn expected_response_type(&self) -> Option<Cow<str>> {
        Some(Cow::Borrowed("application/json"))
    }

    fn deserialize_response(
        &self,
        status_code: StatusCode,
        content_type: Option<&str>,
        buf: &[u8],
    ) -> Self::Response {
        match (status_code, content_type) {
            (code, Some(t)) if code.is_success() && t == "application/json" => {
                Ok(serde_json::from_slice(buf)?)
            }

            (code, None) if code.is_success() => Ok(serde_json::from_slice(buf)?),

            (code, Some(t)) if !code.is_success() && t.starts_with("text/") => {
                let message = std::str::from_utf8(buf).ok().map(|s| s.to_string());
                Err(HttpJsonApiError::UnsuccessfulResponse {
                    status_code: code,
                    message,
                })
            }

            (code, _) => Err(HttpJsonApiError::UnsuccessfulResponse {
                status_code: code,
                message: None,
            }),
        }
    }
}

#[derive(Debug, Error, Display)]
pub enum HttpJsonApiError {
    #[display(fmt = "Error deserializing JSON: {}", _0)]
    InvalidJson(#[from] serde_json::Error),

    #[display(fmt = "Expecting a application/json content type but got: {}", actual)]
    InvalidContentType { actual: String },

    #[display(
        fmt = "HTTP request failed with status code: {}, msg = {:?}",
        status_code,
        message
    )]
    UnsuccessfulResponse {
        status_code: StatusCode,
        message: Option<String>,
    },
}
