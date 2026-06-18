use super::snippet_len;
use crate::{cmd::fuzz::Args, http};
use serde_json::json;

const LATENCY_MULTIPLIER: u64 = 3;
const LATENCY_FLOOR_MS: u64 = 750;

pub async fn payload(
    args: &Args,
    client: reqwest::Client,
    spec: http::RequestSpec,
    base: http::Outcome,
    payload: String,
) -> serde_json::Value {
    let outcome = http::send_once(&client, &spec, snippet_len()).await;
    let reasons = reasons(args, &payload, &outcome, &base);
    json!({
        "payload": payload, "status": outcome.status, "body_len": outcome.body_len,
        "latency_ms": outcome.latency_ms, "interesting": !reasons.is_empty(),
        "reasons": reasons, "snippet": outcome.snippet,
    })
}

fn reasons(
    args: &Args,
    payload: &str,
    outcome: &http::Outcome,
    base: &http::Outcome,
) -> Vec<String> {
    let mut out = Vec::new();
    if outcome.error.is_none() && outcome.status != base.status {
        out.push(format!(
            "status {} != baseline {}",
            outcome.status, base.status
        ));
    }
    if (outcome.body_len as i64 - base.body_len as i64).unsigned_abs() as usize > args.len_threshold
    {
        out.push(format!(
            "len {} vs baseline {}",
            outcome.body_len, base.body_len
        ));
    }
    if outcome.latency_ms > base.latency_ms.saturating_mul(LATENCY_MULTIPLIER) + LATENCY_FLOOR_MS {
        out.push(format!(
            "latency {}ms vs baseline {}ms",
            outcome.latency_ms, base.latency_ms
        ));
    }
    if reflected(payload, outcome, base) {
        out.push("payload reflected in response".into());
    }
    if args
        .match_substr
        .as_ref()
        .is_some_and(|m| outcome.snippet.contains(m))
    {
        out.push(format!("matched '{}'", args.match_substr.as_ref().unwrap()));
    }
    if let Some(error) = &outcome.error {
        out.push(format!("transport error: {error}"));
    }
    out
}

// Baseline subtraction over full body: reflected = present in the response AND
// absent from the baseline body, so a constant body is not a false positive.
fn reflected(payload: &str, outcome: &http::Outcome, base: &http::Outcome) -> bool {
    let needle = payload.as_bytes();
    !needle.is_empty()
        && contains(&outcome.body_raw, needle)
        && !contains(&base.body_raw, needle)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.len() <= haystack.len() && haystack.windows(needle.len()).any(|w| w == needle)
}
