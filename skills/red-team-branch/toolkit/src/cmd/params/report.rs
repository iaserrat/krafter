use crate::cmd;
use serde_json::json;

pub fn emit(base: String, location: String, noisy: bool, found: Vec<serde_json::Value>) {
    let verdict = if found.is_empty() {
        "no hidden parameters reacted".to_string()
    } else {
        format!("{} parameter(s) the server reacts to", found.len())
    };
    cmd::emit(&json!({
        "tool": "params", "url": base, "location": location,
        "noisy_endpoint": noisy, "found": found.len(),
        "verdict": verdict, "parameters": found,
    }));
}
