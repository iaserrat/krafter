mod common;
mod params_fixtures;

fn found_params(v: &serde_json::Value) -> Vec<String> {
    v["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["param"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn params_finds_reflected_hidden_input() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["params", "--url", "/echo"]);
    let found = found_params(&v);
    assert!(found.contains(&"debug".to_string()), "expected 'debug', got {found:?}");
}

// FALSE-POSITIVE ORACLE: an endpoint that intermittently returns 200/503 with a
// constant body has NO reacting parameter. A status-class oracle ungated by the
// noisy flag mints a finding for every probe that lands on the off-status.
#[test]
fn params_clears_flaky_status_endpoint() {
    let port = params_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["params", "--url", "/flaky"]);
    let found = found_params(&v);
    assert!(found.is_empty(), "flaky-status endpoint flagged params: {found:?}");
}

// DETECTION on a noisy endpoint: even with flaky status, a param that genuinely
// reflects its value MUST still be flagged (reflection is independent of status).
#[test]
fn params_flags_reflecting_param_on_flaky_endpoint() {
    let port = params_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["params", "--url", "/reflect"]);
    let found = found_params(&v);
    assert!(found.contains(&"debug".to_string()), "expected reflecting 'debug', got {found:?}");
}
