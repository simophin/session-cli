use bytes::Bytes;
use derive_more::Display;
use http::StatusCode;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum HttpCallError {
    #[display(fmt = "IO error: {}", _0)]
    Io(std::io::Error),

    #[display(fmt = "HTTP error: {}{:?}", _0, _1)]
    HttpErrorCode(StatusCode, Option<String>),
}

pub trait HttpCallSource {
    async fn new_call(
        &self,
        req: http::Request<Bytes>,
    ) -> Result<http::Response<Vec<u8>>, HttpCallError>;
}
