use serde_json::json;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Request};

pub fn capture(request: &mut Request, max_body: u64) -> serde_json::Value {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let headers = headers(request);
    let host_header = header(request, "host");
    let url = split_url(request.url());
    let mut raw = Vec::new();
    let _ = request.as_reader().take(max_body).read_to_end(&mut raw);
    json!({
        "tool": "callback", "ts_ms": ts,
        "remote_addr": request.remote_addr().map(|a| a.to_string()),
        "method": request.method().as_str(), "path": url.path, "query": url.query,
        "host_header": host_header, "headers": headers,
        "body": String::from_utf8_lossy(&raw), "body_len": raw.len(),
    })
}

fn headers(request: &Request) -> Vec<serde_json::Value> {
    request
        .headers()
        .iter()
        .map(|h: &Header| json!({"name": h.field.as_str().as_str(), "value": h.value.as_str()}))
        .collect()
}

fn header(request: &Request, name: &str) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

struct UrlParts {
    path: String,
    query: Option<String>,
}

fn split_url(url: &str) -> UrlParts {
    url.split_once('?')
        .map(|(path, query)| UrlParts {
            path: path.to_string(),
            query: Some(query.to_string()),
        })
        .unwrap_or_else(|| UrlParts {
            path: url.to_string(),
            query: None,
        })
}
