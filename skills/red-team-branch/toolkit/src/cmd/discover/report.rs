use super::{sensitive, signature::Baseline};
use crate::{cmd, http};
use serde_json::json;

pub fn route(path: String, url: String, outcome: http::Outcome) -> serde_json::Value {
    json!({
        "path": path, "url": url, "status": outcome.status,
        "body_len": outcome.body_len, "content_type": outcome.content_type,
        "redirect": outcome.location, "sensitive": sensitive::is_sensitive_path(&path),
    })
}

pub fn emit(base: String, soft404: Baseline, routes: Vec<serde_json::Value>) {
    let sensitive = routes
        .iter()
        .filter(|r| r["sensitive"].as_bool().unwrap_or(false))
        .count();
    let verdict = verdict(routes.len(), sensitive);
    cmd::emit(&json!({
        "tool": "discover", "base": base,
        "soft404": {"status_class": soft404.status_class, "base_len": soft404.len_bucket()},
        "found": routes.len(), "verdict": verdict, "routes": routes,
    }));
}

fn verdict(found: usize, sensitive: usize) -> String {
    if found == 0 {
        "no routes distinguishable from the soft-404 baseline".into()
    } else if sensitive > 0 {
        format!("{found} live route(s); {sensitive} sensitive (admin/debug/config/secrets)")
    } else {
        format!("{found} live route(s) found")
    }
}
