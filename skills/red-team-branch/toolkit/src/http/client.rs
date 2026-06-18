use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone)]
pub struct HttpOpts {
    pub timeout: Duration,
    pub insecure_tls: bool,
    pub proxy: Option<String>,
    pub user_agent: String,
    pub headers: BTreeMap<String, String>,
}

pub fn build_client(opts: &HttpOpts) -> anyhow::Result<Client> {
    let mut builder = Client::builder()
        .timeout(opts.timeout)
        .default_headers(default_headers(&opts.headers))
        .user_agent(opts.user_agent.clone())
        .danger_accept_invalid_certs(opts.insecure_tls)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none());
    if let Some(proxy) = &opts.proxy {
        if !proxy.is_empty() {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?);
        }
    }
    Ok(builder.build()?)
}

fn default_headers(headers: &BTreeMap<String, String>) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (key, value) in headers {
        match (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            (Ok(name), Ok(value)) => {
                map.insert(name, value);
            }
            _ => eprintln!("[rtk][warn] skipping invalid default header '{key}'"),
        }
    }
    map
}
