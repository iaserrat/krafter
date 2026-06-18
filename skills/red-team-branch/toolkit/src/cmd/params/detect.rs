use super::{baseline, CANARY};
use crate::{cmd::cov, http};
use serde_json::json;

const RESPONSE_SNIPPET_LEN: usize = 120;

pub async fn probe(
    client: reqwest::Client,
    baseline: baseline::Baseline,
    name: String,
    spec: http::RequestSpec,
) -> Option<serde_json::Value> {
    let outcome = http::send_once(&client, &spec, RESPONSE_SNIPPET_LEN).await;
    let reasons = reasons(&baseline, &outcome);
    (!reasons.is_empty())
        .then(|| json!({"param": name, "status": outcome.status, "reasons": reasons}))
}

fn reasons(base: &baseline::Baseline, outcome: &http::Outcome) -> Vec<String> {
    let mut out = Vec::new();
    if baseline::reflects(outcome) && !base.reflected {
        out.push("value reflected".to_string());
    }
    if !base.noisy && outcome.error.is_none() && http::status_class(outcome.status) != base.status_class {
        out.push(format!(
            "status {} vs baseline {}xx",
            outcome.status, base.status_class
        ));
    }
    if !base.noisy && cov::len_bucket(outcome.body_len) != base.len_bucket {
        out.push(format!("length class changed ({} bytes)", outcome.body_len));
    }
    if !base.noisy && cov::body_fp(outcome, CANARY, b"") != base.fp {
        out.push("response structure changed".to_string());
    }
    out
}
