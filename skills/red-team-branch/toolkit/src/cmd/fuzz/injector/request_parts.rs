use crate::http::{RequestBody, RequestHeader};

pub fn text_headers(headers: &[(String, String)]) -> Vec<RequestHeader> {
    headers
        .iter()
        .map(|(key, value)| RequestHeader::Text(key.clone(), value.clone()))
        .collect()
}

pub fn raw_header(headers: &[(String, String)], name: &str, value: Vec<u8>) -> Vec<RequestHeader> {
    let mut out = text_headers(headers);
    out.push(RequestHeader::Bytes(name.to_string(), value));
    out
}

pub fn text_body(body: Option<String>) -> RequestBody {
    body.map(RequestBody::Text).unwrap_or(RequestBody::Empty)
}
