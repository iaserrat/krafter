use super::{header_str, Outcome, RequestBody, RequestHeader, RequestSpec, RAW_CAP};
use crate::util;
use reqwest::header::{HeaderValue, CONTENT_TYPE, LOCATION, SERVER};
use reqwest::{Client, Method, RequestBuilder};
use std::time::Instant;

pub async fn send_once(client: &Client, spec: &RequestSpec, snippet_len: usize) -> Outcome {
    let start = Instant::now();
    let Ok(method) = Method::from_bytes(spec.method.to_uppercase().as_bytes()) else {
        return Outcome {
            error: Some(format!("invalid method '{}'", spec.method)),
            ..Default::default()
        };
    };
    let request = match apply_request(client.request(method, spec.url.as_str()), spec) {
        Ok(request) => request,
        Err(error) => {
            return Outcome {
                error: Some(error),
                ..Default::default()
            }
        }
    };
    match request.send().await {
        Ok(resp) => read_response(resp, start, snippet_len).await,
        Err(e) => Outcome {
            latency_ms: start.elapsed().as_millis() as u64,
            latency_us: start.elapsed().as_micros() as u64,
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

fn apply_request(
    mut request: RequestBuilder,
    spec: &RequestSpec,
) -> Result<RequestBuilder, String> {
    for header in &spec.headers {
        request = match header {
            RequestHeader::Text(k, v) => request.header(k.as_str(), v.as_str()),
            RequestHeader::Bytes(k, v) => request.header(k.as_str(), raw_header(v)?),
        };
    }
    Ok(match &spec.body {
        RequestBody::Empty => request,
        RequestBody::Text(body) => request.body(body.clone()),
        RequestBody::Bytes(body) => request.body(body.clone()),
    })
}

pub(crate) fn raw_header(bytes: &[u8]) -> Result<HeaderValue, String> {
    HeaderValue::from_bytes(bytes)
        .map_err(|_| "header value rejected (CR/LF/NUL/DEL/control byte)".into())
}

fn collect_headers(headers: &reqwest::header::HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.as_str().to_string(), v.to_string())))
        .collect()
}

async fn read_response(resp: reqwest::Response, start: Instant, snippet_len: usize) -> Outcome {
    let status = resp.status().as_u16();
    let server = header_str(resp.headers(), SERVER);
    let location = header_str(resp.headers(), LOCATION);
    let content_type = header_str(resp.headers(), CONTENT_TYPE);
    let headers = collect_headers(resp.headers());
    match resp.bytes().await {
        Ok(bytes) => Outcome {
            status,
            latency_ms: start.elapsed().as_millis() as u64,
            latency_us: start.elapsed().as_micros() as u64,
            body_len: bytes.len(),
            body_sha8: util::sha8(&bytes),
            server,
            location,
            content_type,
            snippet: util::truncate(&String::from_utf8_lossy(&bytes), snippet_len),
            error: None,
            body_raw: bytes[..bytes.len().min(RAW_CAP)].to_vec(),
            headers,
        },
        Err(e) => Outcome {
            status,
            server,
            location,
            error: Some(format!("body read: {e}")),
            ..Default::default()
        },
    }
}
