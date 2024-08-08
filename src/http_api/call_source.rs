use super::HttpApi;
use crate::utils::HttpBaseUrl;

pub trait HttpCallSource {
    type Error: std::error::Error;
    type Arg<'a>;

    async fn invoke<Api: HttpApi>(
        &self,
        base: &HttpBaseUrl,
        arg: Self::Arg<'_>,
        api: &Api,
    ) -> Result<Api::Response, Self::Error>;
}
