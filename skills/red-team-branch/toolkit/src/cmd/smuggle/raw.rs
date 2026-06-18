use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;

pub struct RawOutcome {
    pub status: u16,
    pub body_raw: Vec<u8>,
    pub error: Option<String>,
}

const READ_BUF: usize = 65536;
const BODY_CAP: usize = 65536;
const STATUS_LEN: usize = 3;
const SPACE: u8 = b' ';
const CRLFCRLF: &[u8] = b"\r\n\r\n";

pub async fn raw_request(host: &str, port: u16, raw: &[u8], timeout_ms: u64) -> RawOutcome {
    let addr = format!("{host}:{port}");
    let connect = time::timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr));
    let Ok(Ok(mut stream)) = connect.await else {
        return RawOutcome { status: 0, body_raw: vec![], error: Some("connect failed".into()) };
    };
    if stream.write_all(raw).await.is_err() {
        return RawOutcome { status: 0, body_raw: vec![], error: Some("write failed".into()) };
    }
    match read_response(&mut stream, timeout_ms).await {
        Ok(o) => o,
        Err(e) => RawOutcome { status: 0, body_raw: vec![], error: Some(e) },
    }
}

async fn read_response(stream: &mut TcpStream, timeout_ms: u64) -> Result<RawOutcome, String> {
    let dur = Duration::from_millis(timeout_ms);
    let mut buf = vec![0u8; READ_BUF];
    let mut total = Vec::new();
    loop {
        let read = time::timeout(dur, stream.read(&mut buf))
            .await
            .map_err(|_| "read timeout".to_string())?
            .map_err(|e| format!("read error: {e}"))?;
        if read == 0 {
            break;
        }
        total.extend_from_slice(&buf[..read]);
        if total.len() > BODY_CAP || past_body(&total) {
            break;
        }
    }
    if total.is_empty() {
        return Err("empty response".into());
    }
    let status = parse_status(&total);
    Ok(RawOutcome { status, body_raw: total, error: None })
}

fn past_body(data: &[u8]) -> bool {
    find_header_end(data).is_some_and(|hdr_end| body_complete(data, hdr_end))
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(CRLFCRLF.len()).position(|w| w == CRLFCRLF)
}

fn body_complete(data: &[u8], hdr_end: usize) -> bool {
    let body_start = hdr_end + CRLFCRLF.len();
    if body_start >= data.len() {
        return false;
    }
    let headers = std::str::from_utf8(&data[..hdr_end]).unwrap_or("");
    if let Some(cl) = parse_content_length(headers) {
        return data.len() >= body_start + cl;
    }
    true
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        line.strip_prefix("Content-Length:")
            .or_else(|| line.strip_prefix("content-length:"))
            .and_then(|v| v.trim().parse().ok())
    })
}

/// Status code from the response line by finding the first SPACE after the
/// HTTP version token, not a fixed byte offset (versions vary in length).
pub(crate) fn parse_status(data: &[u8]) -> u16 {
    let line = data.split(|&b| b == b'\r' || b == b'\n').next().unwrap_or(data);
    let Some(sp) = line.iter().position(|&b| b == SPACE) else {
        return 0;
    };
    let code = line[sp + 1..].iter().skip_while(|&&b| b == SPACE).take(STATUS_LEN);
    let digits: Vec<u8> = code.copied().collect();
    std::str::from_utf8(&digits)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}
