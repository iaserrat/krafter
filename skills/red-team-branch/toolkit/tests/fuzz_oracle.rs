mod common;
mod fuzz_oracle_fixtures;

// Payload is a substring of /safe's CONSTANT body but the server never echoes
// input. Reflection must be baseline-subtracted, so /safe must NOT be flagged.
const PAYLOAD: &str = "admin";

fn write_wordlist(port: u16) -> String {
    let path = std::env::temp_dir().join(format!("rtk_fuzz_wl_{port}.txt"));
    std::fs::write(&path, format!("{PAYLOAD}\n")).unwrap();
    path.to_string_lossy().into_owned()
}

fn reflected(v: &serde_json::Value) -> bool {
    v["results"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|r| r["reasons"].as_array().unwrap().clone())
        .any(|reason| reason.as_str().unwrap().contains("reflected"))
}

#[test]
fn static_oracle_does_not_flag_nonreflecting_constant_body() {
    let port = fuzz_oracle_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let wl = write_wordlist(port);
    let v = common::run_rtk(&cfg, &[
        "fuzz", "--url", "/safe?q={FUZZ}", "--wordlist", &wl,
    ]);
    assert_eq!(v["mode"], "static");
    assert!(
        !reflected(&v),
        "constant body containing the payload must NOT be flagged reflected: {v}"
    );
    let _ = std::fs::remove_file(&wl);
}

#[test]
fn static_oracle_flags_genuine_reflection() {
    let port = fuzz_oracle_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let wl = write_wordlist(port);
    let v = common::run_rtk(&cfg, &[
        "fuzz", "--url", "/reflect?q={FUZZ}", "--wordlist", &wl,
    ]);
    assert!(
        reflected(&v),
        "server that echoes the payload must be flagged reflected: {v}"
    );
    let _ = std::fs::remove_file(&wl);
}
