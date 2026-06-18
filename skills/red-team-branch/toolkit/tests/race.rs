mod common;
mod race_fixtures;

const COUNT: &str = "20";

// False-positive oracle: an idempotent/locked endpoint answers 200 to every
// concurrent attempt but produces ONE real effect. It MUST be cleared.
#[test]
fn race_clears_locked_idempotent_endpoint() {
    let port = race_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["race", "--url", "/locked", "--count", COUNT]);
    let verdict = v["verdict"].as_str().unwrap();
    assert!(
        v["successes"].as_u64().unwrap() > 1,
        "fixture must return many 2xx so a count-only oracle would trip: {v}"
    );
    assert_eq!(
        v["distinct_successes"].as_u64().unwrap(),
        1,
        "locked endpoint has exactly one distinct effect: {v}"
    );
    assert!(
        !verdict.contains("POSSIBLE RACE"),
        "idempotent endpoint must NOT be flagged: {verdict}"
    );
}

// Detection: a no-lock endpoint double-spends, yielding DISTINCT effects per
// concurrent winner. It MUST be flagged.
#[test]
fn race_flags_double_spend() {
    let port = race_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["race", "--url", "/nolock", "--count", COUNT]);
    let verdict = v["verdict"].as_str().unwrap();
    assert!(
        v["distinct_successes"].as_u64().unwrap() > 1,
        "double-spend yields multiple distinct effects: {v}"
    );
    assert!(
        verdict.contains("POSSIBLE RACE"),
        "double-spend must be flagged: {verdict}"
    );
}
