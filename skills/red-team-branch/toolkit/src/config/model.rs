use crate::http::HttpOpts;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub profiles: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct HttpConfig {
    pub base_url: Option<String>,
    #[serde(default = "super::defaults::timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "super::defaults::concurrency")]
    pub concurrency: usize,
    #[serde(default)]
    pub insecure_tls: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default = "super::defaults::user_agent")]
    pub user_agent: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_ms: super::defaults::timeout_ms(),
            concurrency: super::defaults::concurrency(),
            insecure_tls: false,
            proxy: None,
            user_agent: super::defaults::user_agent(),
            headers: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct SafetyConfig {
    #[serde(default)]
    pub allow_hosts: Vec<String>,
    #[serde(default)]
    pub allow_remote: bool,
}

pub fn http_opts(h: HttpConfig) -> HttpOpts {
    HttpOpts {
        timeout: Duration::from_millis(h.timeout_ms),
        insecure_tls: h.insecure_tls,
        proxy: h.proxy,
        user_agent: h.user_agent,
        headers: h.headers,
    }
}
