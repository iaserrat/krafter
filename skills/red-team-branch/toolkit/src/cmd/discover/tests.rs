use super::sensitive::is_sensitive_path;
use super::signature::Baseline;
use crate::http::Outcome;

fn out(status: u16, body_len: usize) -> Outcome {
    Outcome { status, body_len, ..Default::default() }
}

#[test]
fn dynamic_length_catch_all_stays_indistinguishable() {
    // 2xx baseline with a wide length band (short vs long echoed path).
    let base = Baseline::calibrate(&out(200, 110), &out(200, 614));
    // A mid-length 200 body must read as the same catch-all, not a new route.
    assert!(base.indistinguishable(&out(200, 314)));
}

#[test]
fn distinct_status_class_is_not_baseline() {
    // 404 baseline: a 200 is a different status class -> a real route.
    let base = Baseline::calibrate(&out(404, 24), &out(404, 24));
    assert!(!base.indistinguishable(&out(200, 24)));
    // Same-class 401 DOES collapse here (all 4xx share a class); the denied
    // short-circuit in probe() is what surfaces it, asserted in tests/discover.rs.
    assert!(base.indistinguishable(&out(401, 24)));
}

#[test]
fn flags_hot_paths_only() {
    for path in [
        "admin",
        "wp-admin",
        ".env",
        ".git/config",
        "actuator/health",
        "backup.zip",
    ] {
        assert!(is_sensitive_path(path), "{path} should be sensitive");
    }
    for path in ["robots.txt", "login", "status", ".well-known/security.txt"] {
        assert!(!is_sensitive_path(path), "{path} should not be sensitive");
    }
}
