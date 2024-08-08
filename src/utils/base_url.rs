use derive_more::{AsRef, Deref};
use reqwest::IntoUrl;
use url::Url;

#[derive(Deref, AsRef)]
pub struct HttpBaseUrl(Url);

impl HttpBaseUrl {
    pub fn new(url: impl IntoUrl) -> Option<Self> {
        let url = url.into_url().ok()?;

        if (url.scheme().eq_ignore_ascii_case("http") || url.scheme().eq_ignore_ascii_case("https"))
            && url.has_host()
            && !url.has_authority()
            && url.query().is_none()
            && url.fragment().is_none()
        {
            Some(Self(url))
        } else {
            None
        }
    }

    pub fn build_upon(&self) -> HttpUrlBuilder {
        HttpUrlBuilder(self.0.clone())
    }
}

pub struct HttpUrlBuilder(Url);

impl HttpUrlBuilder {
    pub fn append_path(mut self, path_segment: &str) -> Self {
        self.0
            .path_segments_mut()
            .expect("To have path segment")
            .push(path_segment);
        self
    }

    pub fn append_query(mut self, query_name: &str, query_value: &str) -> Self {
        self.0
            .query_pairs_mut()
            .append_pair(query_name, query_value);
        self
    }

    pub fn build(self) -> Url {
        self.0
    }
}
