use serde_json::json;
use std::collections::BTreeMap;

// A race needs MORE THAN ONE distinct successful effect; one effect = idempotent.
const DISTINCT_EFFECT_THRESHOLD: usize = 1;

pub(super) struct Verdict<'a> {
    pub url: &'a str,
    pub success_range: &'a str,
    pub fired: usize,
    pub successes: usize,
    pub distinct_successes: usize,
    pub by_status: BTreeMap<u16, usize>,
    pub latencies: &'a [u64],
}

pub(super) fn emit(v: Verdict) {
    let verdict = verdict(v.successes, v.distinct_successes);
    super::super::emit(&json!({
        "tool": "race",
        "url": v.url,
        "fired": v.fired,
        "success_range": v.success_range,
        "successes": v.successes,
        "distinct_successes": v.distinct_successes,
        "by_status": v.by_status.into_iter().map(|(k, c)| (k.to_string(), c)).collect::<BTreeMap<_, _>>(),
        "latency": crate::util::stats(v.latencies),
        "verdict": verdict,
    }));
}

fn verdict(successes: usize, distinct: usize) -> String {
    if distinct > DISTINCT_EFFECT_THRESHOLD {
        format!("POSSIBLE RACE - {successes} succeeded with {distinct} DISTINCT effects; if only one should, this is a TOCTOU/missing-lock bug")
    } else {
        format!("no race observed ({successes} succeeded, {distinct} distinct effect); endpoint looks idempotent/locked")
    }
}
