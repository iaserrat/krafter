use super::identities::Identity;
use super::oracle::{self, Probe};
use crate::{cmd, cmd::cov, http};
use serde_json::json;

pub fn cell(method: String, identity: String, outcome: http::Outcome) -> serde_json::Value {
    json!({
        "method": method,
        "identity": identity,
        "status": outcome.status,
        "body_len": outcome.body_len,
        "body_sha8": outcome.body_sha8,
        "blocked": cov::block_reason(&outcome),
    })
}

pub fn emit(
    url: String,
    methods: Vec<String>,
    identities: Vec<Identity>,
    cells: Vec<serde_json::Value>,
) {
    let findings = findings(&cells);
    let verdict = if findings.is_empty() {
        "no function-level authz gap observed".to_string()
    } else {
        format!(
            "FUNCTION-LEVEL AUTHZ GAP: {} lower-privileged state-changing call(s)",
            findings.len()
        )
    };
    cmd::emit(&json!({
        "tool": "matrix", "url": url, "methods": methods,
        "identities": identities.into_iter().map(|i| i.label).collect::<Vec<_>>(),
        "verdict": verdict, "findings": findings, "matrix": cells,
    }));
}

fn findings(cells: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let probes: Vec<Probe> = cells.iter().filter_map(oracle::parse).collect();
    probes
        .iter()
        .filter(|p| oracle::is_gap(p, &probes))
        .map(|p| json!({
            "method": p.method, "identity": p.identity, "status": p.status,
            "note": "lower-privileged identity reaches a state-changing method the endpoint gates",
        }))
        .collect()
}
