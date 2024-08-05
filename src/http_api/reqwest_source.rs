use crate::http_api::HttpCallError;
use bytes::Bytes;
use http::{Request, Response};

impl super::HttpCallSource for reqwest::Client {
    async fn new_call(&self, req: Request<Bytes>) -> Result<Response<Vec<u8>>, HttpCallError> {
        let (parts, body) = req.into_parts();

        todo!()
    }
}
