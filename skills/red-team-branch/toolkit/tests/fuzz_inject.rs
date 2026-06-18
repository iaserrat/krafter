mod common;
mod fuzz_inject_fixtures;

use serde_json::Value;

// A reflecting fixture flags every channel; the test then asserts the emitted
// repro_curl / request faithfully reproduces what the channel actually sent.
fn run(cfg: &str, extra: &[&str]) -> Value {
    let mut args = vec!["fuzz", "--mutate", "--seed", "7", "--max-exec", "400"];
    args.extend_from_slice(extra);
    common::run_rtk(cfg, &args)
}

fn anomalies(v: &Value) -> &Vec<Value> {
    v["anomalies"].as_array().expect("anomalies array")
}

fn first_repro(v: &Value) -> String {
    anomalies(v)[0]["repro_curl"].as_str().expect("repro_curl").to_string()
}

// MEDIUM (correctness): the url channel sends method/headers/body, but the repro
// dropped them — a copied PoC issued a plain GET with no auth header and no body.
#[test]
fn url_repro_reproduces_method_headers_and_body() {
    let port = fuzz_inject_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = run(&cfg, &[
        "--url", "/reflect?q={FUZZ}",
        "--method", "POST",
        "--header", "X-Token: secret",
        "--body", "auth=1",
    ]);
    assert_eq!(v["channel"], "url");
    assert!(!anomalies(&v).is_empty(), "reflecting url channel should flag: {v}");
    let repro = first_repro(&v);
    assert!(repro.contains("-X POST"), "repro dropped method: {repro}");
    assert!(repro.contains("X-Token: secret"), "repro dropped header: {repro}");
    assert!(repro.contains("auth=1"), "repro dropped body: {repro}");
}

// Header channel must flag a reflected value and repro the right method/header.
#[test]
fn header_channel_flags_and_repros() {
    let port = fuzz_inject_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = run(&cfg, &[
        "--url", "/header",
        "--method", "POST",
        "--header", "X-Probe: {FUZZ}",
    ]);
    assert_eq!(v["channel"], "header");
    assert!(!anomalies(&v).is_empty(), "reflecting header channel should flag: {v}");
    let repro = first_repro(&v);
    assert!(repro.contains("-X POST") && repro.contains("X-Probe:"), "header repro wrong: {repro}");
}

// MEDIUM (quality): multipart silently discarded the --body template. The file
// part (echoed by /echo) must carry the JSON wrapper, and so must the repro.
#[test]
fn multipart_uses_body_template() {
    let port = fuzz_inject_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = run(&cfg, &[
        "--url", "/echo",
        "--method", "POST",
        "--channel", "multipart",
        "--body", r#"{"name":"{FUZZ}"}"#,
    ]);
    assert_eq!(v["channel"], "multipart");
    assert!(!anomalies(&v).is_empty(), "reflecting multipart channel should flag: {v}");
    let snippet = anomalies(&v)[0]["snippet"].as_str().expect("snippet");
    assert!(snippet.contains(r#"{"name":"#), "part dropped --body wrapper: {snippet}");
    // repro encodes the part as printf hex bytes; {"name = 7b 22 6e 61 6d 65.
    let repro = first_repro(&v);
    assert!(repro.contains(r"\x7b\x22\x6e\x61\x6d\x65"), "repro dropped wrapper: {repro}");
}
