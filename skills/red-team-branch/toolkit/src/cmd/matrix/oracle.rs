//! Privilege-differential oracle: a function-level authz gap is a *lower-priv*
//! actor reaching a state-changing verb that the endpoint actually gates.
use crate::http;

const DEFAULT_IDENTITY: &str = "default";
const STATE_CHANGING: &[&str] = &["POST", "PUT", "PATCH", "DELETE"];

pub struct Probe {
    pub method: String,
    pub identity: String,
    pub status: u16,
}

pub fn parse(cell: &serde_json::Value) -> Option<Probe> {
    Some(Probe {
        method: cell["method"].as_str()?.to_string(),
        identity: cell["identity"].as_str()?.to_string(),
        status: cell["status"].as_u64()? as u16,
    })
}

/// Flag iff a lower-priv actor reaches a state-changing verb AND the endpoint is
/// access-controlled: the authorized identity reaches the same verb, and the
/// actor is denied somewhere (proving auth is enforced, not public-by-design).
pub fn is_gap(p: &Probe, probes: &[Probe]) -> bool {
    p.identity != DEFAULT_IDENTITY
        && STATE_CHANGING.contains(&p.method.as_str())
        && http::reached_operation(p.status)
        && authorized_reaches(&p.method, probes)
        && actor_is_gated(&p.identity, probes)
}

fn authorized_reaches(method: &str, probes: &[Probe]) -> bool {
    probes.iter().any(|q| {
        q.identity == DEFAULT_IDENTITY && q.method == method && http::reached_operation(q.status)
    })
}

fn actor_is_gated(identity: &str, probes: &[Probe]) -> bool {
    probes
        .iter()
        .any(|q| q.identity == identity && http::is_denied(q.status))
}
