use base64::Engine as _;
use std::fmt::Write as _;

pub fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn b64dec(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

pub fn escape_printf(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 4);
    for &byte in bytes {
        let _ = write!(out, "\\x{byte:02x}");
    }
    out
}
