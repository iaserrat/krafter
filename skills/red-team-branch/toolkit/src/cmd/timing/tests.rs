use super::signal::{detect, Variant};

fn v(stddev: f64, count: usize) -> Variant {
    Variant { stddev, count }
}

// Constant-time: zero delta with real noise must NOT flag (false-positive guard).
#[test]
fn no_delta_with_noise_clears() {
    assert!(!detect(0.0, v(50.0, 60), v(50.0, 60)));
}

// A sub-ms delta (~400us) that clears 3 combined SEs at n=60 must flag.
#[test]
fn submillisecond_delta_over_noise_flags() {
    assert!(detect(400.0, v(50.0, 60), v(50.0, 60)));
}

// Edge case the old rule failed: identical samples (stddev=0) with a real delta.
#[test]
fn zero_variance_nonzero_delta_flags() {
    assert!(detect(120.0, v(0.0, 30), v(0.0, 30)));
}

// Zero variance AND zero delta is not a signal.
#[test]
fn zero_variance_zero_delta_clears() {
    assert!(!detect(0.0, v(0.0, 30), v(0.0, 30)));
}

// More samples tighten the threshold: a delta inside noise at n=10 clears, but
// the same delta with the same per-sample noise crosses the bar at n=400.
#[test]
fn threshold_tightens_with_sample_count() {
    assert!(!detect(60.0, v(120.0, 10), v(120.0, 10)));
    assert!(detect(60.0, v(120.0, 400), v(120.0, 400)));
}
