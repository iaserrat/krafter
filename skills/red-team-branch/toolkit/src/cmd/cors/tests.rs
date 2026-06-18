use super::probe::{build_origins, CorsResult};
use super::report::classify;

fn res(origin: &str, acao: Option<&str>, acac: Option<&str>) -> CorsResult {
    CorsResult {
        origin: origin.into(),
        acao: acao.map(Into::into),
        acac: acac.map(Into::into),
        acam: None,
        status: 200,
    }
}

#[test]
fn default_origins_include_evil_null_subdomain() {
    let origins = build_origins(&[]);
    assert!(origins.iter().any(|o| o == "https://evil.com"));
    assert!(origins.iter().any(|o| o == "null"));
    assert!(origins.len() >= 3);
}

#[test]
fn custom_origins_replace_defaults() {
    let origins = build_origins(&["https://x.com".to_string()]);
    assert_eq!(origins, vec!["https://x.com"]);
}

#[test]
fn classify_flags_reflected_origin() {
    let r = res("https://evil.com", Some("https://evil.com"), None);
    assert!(classify(&r).contains(&"origin-reflected".to_string()));
}

#[test]
fn classify_flags_wildcard_with_credentials() {
    let r = res("https://x.com", Some("*"), Some("true"));
    let issues = classify(&r);
    assert!(issues.contains(&"wildcard-with-credentials".to_string()));
    assert!(issues.contains(&"credentials-allowed".to_string()));
}

#[test]
fn classify_flags_null_origin() {
    let r = res("null", Some("null"), None);
    assert!(classify(&r).contains(&"null-origin-accepted".to_string()));
}

#[test]
fn safe_cors_produces_no_issues() {
    let r = res("https://evil.com", Some("https://safe.com"), None);
    assert!(classify(&r).is_empty());
}

// Negative control: a safe own-origin allowlist that never reflects the
// attacker is inert even with ACAC:true; it must be CLEARED.
#[test]
fn safe_allowlist_with_credentials_cleared() {
    let r = res("https://evil.com", Some("https://app.example.com"), Some("true"));
    assert!(classify(&r).is_empty(), "issues: {:?}", classify(&r));
}

// Reflected attacker origin + credentials is the real vuln; must be FLAGGED.
#[test]
fn reflected_origin_with_credentials_flagged() {
    let r = res("https://evil.com", Some("https://evil.com"), Some("true"));
    let issues = classify(&r);
    assert!(issues.contains(&"origin-reflected".to_string()));
    assert!(issues.contains(&"credentials-allowed".to_string()));
}

// Preflight negative control: OPTIONS that grants no method (no ACAM) is not
// "accepted" regardless of status; preflight positive needs ACAM + ACAO.
#[test]
fn preflight_requires_acam_and_acao() {
    use super::report::preflight_accepted;
    let mut safe = res("https://evil.com", Some("https://app.example.com"), None);
    safe.acam = Some("GET, POST".into());
    assert!(!preflight_accepted(&safe, "DELETE"));

    let mut vuln = res("https://evil.com", Some("https://evil.com"), None);
    vuln.acam = Some("GET, DELETE".into());
    assert!(preflight_accepted(&vuln, "DELETE"));
}
