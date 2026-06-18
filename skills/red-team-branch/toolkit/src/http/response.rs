use reqwest::header::{HeaderMap, HeaderName};
use serde::Serialize;

pub const RAW_CAP: usize = 64 * 1024;
pub const NO_SNIPPET_LEN: usize = 0;

#[derive(Serialize, Clone, Default)]
pub struct Outcome {
    pub status: u16,
    pub latency_ms: u64,
    // Microsecond latency; sub-ms side-channels truncate to 0 in latency_ms.
    pub latency_us: u64,
    pub body_len: usize,
    pub body_sha8: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip)]
    pub body_raw: Vec<u8>,
    // Full response headers (name, value), duplicates preserved (e.g. Set-Cookie).
    #[serde(skip)]
    pub headers: Vec<(String, String)>,
}

pub fn header_str(headers: &HeaderMap, name: HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// First response header matching `name` case-insensitively.
pub fn find_header(out: &Outcome, name: &str) -> Option<String> {
    out.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
}
