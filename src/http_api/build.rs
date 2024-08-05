use url::Url;

use super::HttpApi;

pub fn build_http_url(api: &impl HttpApi, base_url: &Url) -> Result<Url, url::ParseError> {
    let mut url = base_url.clone();
    for segment in api.path_segments() {
        url = url.join(&segment)?;
    }

    for (query_name, query_value) in api.queries() {
        url.query_pairs_mut().append_pair(&query_name, &query_value);
    }

    Ok(url)
}
