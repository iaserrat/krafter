//! Curl-flag rendering shared by repros so an emitted PoC carries the same
//! method, static headers, and body the channel actually sends.

pub fn header_flags(headers: &[(String, String)]) -> String {
    headers
        .iter()
        .map(|(name, value)| format!(" -H '{name}: {value}'"))
        .collect()
}

pub fn body_flag(body: &Option<String>) -> String {
    match body {
        Some(value) => format!(" --data '{value}'"),
        None => String::new(),
    }
}
