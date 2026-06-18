use std::fmt::Write as _;

#[derive(Clone)]
pub enum RequestBody {
    Empty,
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone)]
pub enum RequestHeader {
    Text(String, String),
    Bytes(String, Vec<u8>),
}

#[derive(Clone)]
pub struct RequestSpec {
    pub method: String,
    pub url: String,
    pub headers: Vec<RequestHeader>,
    pub body: RequestBody,
}

impl RequestSpec {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: Vec::new(),
            body: RequestBody::Empty,
        }
    }

    pub fn with_text_headers(mut self, headers: &[(String, String)]) -> Self {
        self.headers = headers
            .iter()
            .map(|(k, v)| RequestHeader::Text(k.clone(), v.clone()))
            .collect();
        self
    }

    pub fn with_body(mut self, body: Option<String>) -> Self {
        self.body = body.map(RequestBody::Text).unwrap_or(RequestBody::Empty);
        self
    }

    pub fn render(&self, token: &str, value: &str) -> RequestSpec {
        let ph = format!("{{{token}}}");
        let rep = |s: &str| s.replace(&ph, value);
        RequestSpec {
            method: self.method.clone(),
            url: rep(&self.url),
            headers: self.headers.iter().map(|h| h.render(&rep)).collect(),
            body: self.body.render(&rep),
        }
    }
}

impl RequestHeader {
    fn render(&self, rep: &impl Fn(&str) -> String) -> Self {
        match self {
            RequestHeader::Text(k, v) => RequestHeader::Text(k.clone(), rep(v)),
            RequestHeader::Bytes(k, v) => RequestHeader::Bytes(k.clone(), v.clone()),
        }
    }
}

impl RequestBody {
    fn render(&self, rep: &impl Fn(&str) -> String) -> Self {
        match self {
            RequestBody::Empty => RequestBody::Empty,
            RequestBody::Text(v) => RequestBody::Text(rep(v)),
            RequestBody::Bytes(v) => RequestBody::Bytes(v.clone()),
        }
    }
}

pub fn pct_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 3);
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}
