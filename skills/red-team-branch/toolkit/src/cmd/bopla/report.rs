use super::{
    probe::{ProbeResult, Status},
    request::Target,
};
use crate::cmd;
use serde_json::{json, Value};

fn label(status: &Status) -> &'static str {
    match status {
        Status::Accepted => "accepted",
        Status::Cleared => "cleared",
        Status::Inconclusive => "inconclusive",
    }
}

pub fn result(key: &str, value: &Value, result: &ProbeResult) -> serde_json::Value {
    json!({"field": key, "value": value, "write_status": result.write_status, "read_status": result.read_status, "status": label(&result.status)})
}

pub fn emit(
    as_profile: Option<String>,
    target: Target,
    accepted: Vec<String>,
    results: Vec<serde_json::Value>,
) {
    let verdict = if accepted.is_empty() {
        "no candidate field persisted — mass assignment not observed for these keys".to_string()
    } else {
        format!(
            "MASS ASSIGNMENT: persisted privileged field(s): {}",
            accepted.join(", ")
        )
    };
    cmd::emit(&json!({
        "tool": "bopla", "write_url": target.write_url, "read_url": target.read_url,
        "as": as_profile, "verdict": verdict, "accepted_fields": accepted, "results": results,
    }));
}
