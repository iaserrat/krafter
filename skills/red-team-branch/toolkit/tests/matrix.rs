mod common;
mod matrix_fixtures;

const METHODS: &str = "GET,POST,PUT,PATCH,DELETE";

// Detection: a gated endpoint (anon denied on GET) that anon can still DELETE.
#[test]
fn matrix_flags_anon_delete() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(
        &cfg,
        &["matrix", "--url", "/records/1", "--methods", "GET,DELETE"],
    );
    let findings = v["findings"].as_array().unwrap();
    assert!(findings
        .iter()
        .any(|f| f["method"] == "DELETE" && f["identity"] == "anon"));
}

// False-positive oracle: public-by-design mutating endpoint must be CLEARED.
#[test]
fn matrix_clears_public_endpoint() {
    let port = matrix_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["matrix", "--url", "/public/1", "--methods", METHODS]);
    let findings = v["findings"].as_array().unwrap();
    assert!(
        findings.is_empty(),
        "public endpoint must not be flagged, got {findings:?}"
    );
}

// Detection incl. 3xx: anon can DELETE (200) and POST (303) a gated endpoint.
#[test]
fn matrix_flags_gated_leak_including_3xx() {
    let port = matrix_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["matrix", "--url", "/gated/1", "--methods", METHODS]);
    let findings = v["findings"].as_array().unwrap();
    let anon = |m: &str| {
        findings
            .iter()
            .any(|f| f["method"] == m && f["identity"] == "anon")
    };
    assert!(anon("DELETE"), "anon DELETE (200) is a gap: {findings:?}");
    assert!(anon("POST"), "anon POST (303 redirect) is a gap: {findings:?}");
    assert!(!anon("PUT"), "anon PUT is 401-denied, not a gap: {findings:?}");
}
