use super::probe;
use super::raw::{parse_status, RawOutcome};
use super::report::classify;

#[test]
fn cl_te_has_conflicting_headers() {
    let t = probe::all_techniques("localhost:8080", "/", "POST", "/canary", &[]);
    let cl_te = t.iter().find(|x| x.name == "CL.TE").unwrap();
    let has_cl = cl_te.header_lines.iter().any(|l| l.starts_with("Content-Length:"));
    let has_te = cl_te.header_lines.iter().any(|l| l.starts_with("Transfer-Encoding:"));
    assert!(has_cl, "CL.TE must have Content-Length");
    assert!(has_te, "CL.TE must have Transfer-Encoding");
    let body_str = String::from_utf8_lossy(&cl_te.body);
    assert!(body_str.contains("0\r\n\r\n"), "must contain chunked terminator");
    assert!(body_str.contains("GET /canary"), "must contain smuggled prefix");
}

#[test]
fn all_techniques_include_smuggled_prefix() {
    let techniques = probe::all_techniques("h:80", "/x", "GET", "/c", &[]);
    for t in &techniques {
        let body = String::from_utf8_lossy(&t.body);
        assert!(body.contains("GET /c"), "{} missing smuggled prefix", t.name);
    }
    let names: Vec<&str> = techniques.iter().map(|t| t.name).collect();
    assert!(names.contains(&"CL.TE"));
    assert!(names.contains(&"TE.CL"));
    assert!(names.contains(&"TE.TE"));
}

#[test]
fn probe_build_produces_valid_raw_request() {
    let tech = probe::all_techniques("host:80", "/x", "POST", "/c", &[])
        .into_iter().find(|t| t.name == "CL.TE").unwrap();
    let probe = probe::Probe::build(tech);
    let raw = String::from_utf8_lossy(&probe.raw_request);
    assert!(raw.starts_with("POST /x HTTP/1.1\r\n"), "must start with request line, got: {raw:?}");
    assert!(raw.contains("Host: host:80\r\n"), "must contain Host header");
    assert!(raw.ends_with("GET /c HTTP/1.1\r\nHost: host:80\r\n\r\n"), "must end with smuggled prefix");
}

#[test]
fn parse_status_tolerates_version_and_spacing() {
    assert_eq!(parse_status(b"HTTP/1.1 200 OK\r\n"), 200);
    assert_eq!(parse_status(b"HTTP/1.0 404 Not Found\r\n"), 404);
    // Fixed-offset parsing broke on these: shorter version / extra space.
    assert_eq!(parse_status(b"HTTP/2 503 Unavailable\r\n"), 503);
    assert_eq!(parse_status(b"HTTP/1.1  301 Moved\r\n"), 301);
    assert_eq!(parse_status(b"garbage"), 0);
}

fn outcome(status: u16, body: &str) -> RawOutcome {
    RawOutcome { status, body_raw: body.as_bytes().to_vec(), error: None }
}

#[test]
fn classify_flags_only_canary_reflection_not_status_flip() {
    let t = probe::all_techniques("h:80", "/", "POST", "/c", &[]).pop().unwrap();
    // Follow-up status differs from a 200 baseline but no canary: must be SAFE.
    let flip = classify(&outcome(400, "x"), &outcome(503, "overloaded"), &t, "/c");
    assert_eq!(flip.verdict, "SAFE", "bare status flip must not be flagged");
    // Smuggled canary surfaced in the follow-up body: the one sound signal.
    let hit = classify(&outcome(400, "x"), &outcome(200, "served /c here"), &t, "/c");
    assert_eq!(hit.verdict, "EXPLOITABLE");
}
