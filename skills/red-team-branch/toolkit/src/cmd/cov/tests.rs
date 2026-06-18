use super::*;
use crate::http::Outcome;

fn outcome(status: u16, body: &[u8]) -> Outcome {
    Outcome {
        status,
        body_len: body.len(),
        body_raw: body.to_vec(),
        ..Default::default()
    }
}

#[test]
fn novelty_ignores_dynamic_noise_but_catches_structure() {
    let canary = "rtkdeadbeef";
    let a = outcome(200, br#"{"id":"a1b2c3","ok":true}"#);
    let b = outcome(200, br#"{"id":"f9e8d7","ok":true}"#);
    let c = outcome(200, br#"{"error":"boom","ok":false,"detail":"x"}"#);
    assert_eq!(novelty_key(&a, canary, b""), novelty_key(&b, canary, b""));
    assert_ne!(novelty_key(&a, canary, b""), novelty_key(&c, canary, b""));
}

#[test]
fn errsig_requires_absent_in_baseline() {
    let base = outcome(200, b"hello");
    let hit = outcome(200, b"... You have an error in your SQL syntax ...");
    assert!(oracle_mask(&hit, &base, "rtk0", b"'", 9999.0) & B_ERRSIG != 0);
    let base2 = outcome(200, b"You have an error in your SQL syntax baseline");
    assert!(oracle_mask(&hit, &base2, "rtk0", b"'", 9999.0) & B_ERRSIG == 0);
}

#[test]
fn reflection_detected_on_raw_bytes() {
    let base = outcome(200, b"ok");
    let payload = b"<svg/onload=1>";
    let hit = outcome(200, b"echo: <svg/onload=1> done");
    assert!(oracle_mask(&hit, &base, "rtk0", payload, 9999.0) & B_REFLECT != 0);
}

// Baseline subtraction: a constant body that merely CONTAINS the payload
// substring (server never echoes input) must NOT be flagged as reflection.
#[test]
fn reflection_requires_absent_in_baseline() {
    let payload = b"admin";
    let constant = b"welcome, admin panel"; // present in BOTH base and hit
    let base = outcome(200, constant);
    let hit = outcome(200, constant);
    assert!(
        oracle_mask(&hit, &base, "rtk0", payload, 9999.0) & B_REFLECT == 0,
        "non-reflecting constant body must not set B_REFLECT"
    );
    // A truly reflecting server adds the payload that was absent from baseline.
    let base2 = outcome(200, b"welcome");
    let hit2 = outcome(200, b"welcome admin");
    assert!(
        oracle_mask(&hit2, &base2, "rtk0", payload, 9999.0) & B_REFLECT != 0,
        "genuine reflection must set B_REFLECT"
    );
}
