use super::signal::Variant;
use crate::util::Stats;
use serde_json::json;

const ROUND_2_SCALE: f64 = 100.0;

// Samples are microseconds (sub-ms side-channels truncate to 0 in ms); the Stats
// `_ms` fields therefore carry microsecond values here.
pub(super) fn emit(url_template: &str, a_value: &str, b_value: &str, a: Stats, b: Stats) {
    let median_delta = a.median_ms - b.median_ms;
    let signal = super::signal::detect(median_delta, variant(&a), variant(&b));
    super::super::emit(&json!({
        "tool": "timing",
        "url_template": url_template,
        "a_value": a_value,
        "b_value": b_value,
        "units": "microseconds",
        "a": a,
        "b": b,
        "median_delta_us": round2(median_delta),
        "signal": signal,
        "note": note(signal),
    }));
}

fn variant(s: &Stats) -> Variant {
    Variant { stddev: s.stddev_ms, count: s.count }
}

fn note(signal: bool) -> &'static str {
    if signal {
        "median delta exceeds 3 combined standard errors of the median - a timing side-channel is plausible; raise --samples to tighten"
    } else {
        "no significant timing signal at this sample size"
    }
}

fn round2(x: f64) -> f64 {
    (x * ROUND_2_SCALE).round() / ROUND_2_SCALE
}
