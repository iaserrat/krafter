mod common;
mod timing_fixtures;

// Body carries the {VAR} placeholder; a_value is the valid (slow-leak) input,
// b_value an invalid one. Higher --samples tightens the SE-of-median test.
fn args(path: &'static str) -> Vec<&'static str> {
    let body: &'static str = r#"{"user":"{VAR}"}"#;
    let mut a = vec!["timing", "--url", path];
    a.extend([
        "--body", body,
        "--a-value", "alice",
        "--b-value", "zzzzz",
        "--samples", "60",
    ]);
    a
}

fn signal(v: &serde_json::Value) -> bool {
    v["signal"].as_bool().expect("signal bool present")
}

// FALSE-POSITIVE ORACLE: a constant-time endpoint does identical sub-ms work for
// every value. The probe MUST clear it (signal=false).
#[test]
fn timing_clears_constant_time_endpoint() {
    let port = timing_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &args("/login"));
    assert!(!signal(&v), "constant-time endpoint flagged: {v}");
}

// DETECTION: a valid-only sub-millisecond leak (~hundreds of microseconds) MUST
// be flagged. ms-resolution truncates the delta to 0 and never fires this.
#[test]
fn timing_flags_submillisecond_leak() {
    let port = timing_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &args("/enum"));
    assert!(signal(&v), "sub-ms timing leak not flagged: {v}");
    // The signal lives in microseconds; ms-resolution would truncate this delta.
    let delta_us = v["median_delta_us"].as_f64().expect("median_delta_us present");
    assert!(delta_us.abs() > 0.0, "delta collapsed to 0 (ms truncation?): {v}");
}
