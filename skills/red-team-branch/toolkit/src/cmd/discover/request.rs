use crate::http;

pub fn get(url: &str) -> http::RequestSpec {
    http::RequestSpec::new("GET", url)
}
