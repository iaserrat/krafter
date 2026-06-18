use crate::{cmd, http};
use serde_json::json;

pub fn emit(
    url: String,
    payloads: &[String],
    base: &http::Outcome,
    findings: Vec<serde_json::Value>,
) {
    let interesting = findings
        .iter()
        .filter(|f| f["interesting"].as_bool().unwrap_or(false))
        .count();
    cmd::emit(&json!({
        "tool": "fuzz",
        "mode": "static",
        "url_template": url,
        "baseline": {"status": base.status, "body_len": base.body_len, "latency_ms": base.latency_ms},
        "tested": payloads.len(),
        "interesting_count": interesting,
        "results": findings,
    }));
}
