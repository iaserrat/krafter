mod common;

#[test]
fn headers_reports_missing_security_headers() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "headers", "--url", "/admin",
    ]);
    assert_eq!(v["probe"], "headers");
    let issues = v["issues"].as_array().unwrap();
    assert!(!issues.is_empty(), "should report missing security headers");
    assert!(issues.iter().any(|i| i.as_str().unwrap() == "missing-hsts"));
}

#[test]
fn headers_on_404_still_checks() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "headers", "--url", "/nope",
    ]);
    assert_eq!(v["probe"], "headers");
}

#[test]
fn headers_sees_present_headers_and_insecure_cookie() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "headers", "--url", "/secure", "--path", "",
    ]);
    let issues: Vec<&str> = v["issues"].as_array().unwrap()
        .iter().map(|i| i.as_str().unwrap()).collect();
    // Headers are actually present -> must NOT be reported missing.
    assert!(!issues.contains(&"missing-hsts"), "issues: {issues:?}");
    assert!(!issues.contains(&"missing-csp"), "issues: {issues:?}");
    // The Set-Cookie header is read, and its missing flags are caught.
    assert_eq!(v["cookie_count"], 1);
    assert!(issues.iter().any(|i| i.contains("missing-secure")), "issues: {issues:?}");
}

// Transport failure must not be reported as a fully-vulnerable header posture.
#[test]
fn headers_on_closed_port_does_not_report_missing() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &["headers", "--url", "/"]);
    assert_eq!(v["probe"], "headers");
    assert_eq!(v["verdict"], "HEADERS ERROR", "verdict: {}", v["verdict"]);
    // No missing-* claims may be fabricated from a server we never reached.
    assert!(v["issues"].is_null(), "error doc must not carry issues: {v}");
    assert!(v["error"].is_string(), "error doc must report the transport error: {v}");
}