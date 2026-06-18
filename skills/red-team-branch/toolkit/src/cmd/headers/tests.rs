#[cfg(test)]
mod tests {
    use super::super::check::{check_cookies, check_security_headers, CookieCheck};
    use super::super::cookie::parse_cookie;
    use std::collections::BTreeMap;

    #[test]
    fn cookie_missing_flags_reported() {
        let cookies = vec![CookieCheck {
            name: "session".into(), secure: false, http_only: false, same_site: None,
        }];
        let mut issues = Vec::new();
        check_cookies(&cookies, &mut issues);
        assert!(issues.iter().any(|i| i.contains("secure")));
        assert!(issues.iter().any(|i| i.contains("httponly")));
        assert!(issues.iter().any(|i| i.contains("samesite")));
    }

    #[test]
    fn cookie_with_all_flags_clean() {
        let cookies = vec![CookieCheck {
            name: "session".into(), secure: true, http_only: true, same_site: Some("lax".into()),
        }];
        let mut issues = Vec::new();
        check_cookies(&cookies, &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn missing_security_headers_flagged() {
        let headers = BTreeMap::new();
        let mut issues = Vec::new();
        check_security_headers(&headers, &mut issues);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i == "missing-hsts"));
    }

    // Value containing "secure"/"httponly" as substring must NOT set the flags.
    #[test]
    fn parse_cookie_ignores_substring_in_value() {
        let c = parse_cookie("sid=insecure-httponly-token");
        assert_eq!(c.name, "sid");
        assert!(!c.secure, "value substring must not set Secure");
        assert!(!c.http_only, "value substring must not set HttpOnly");
        assert!(c.same_site.is_none(), "value substring must not set SameSite");
    }

    // Genuine attribute tokens must be parsed structurally.
    #[test]
    fn parse_cookie_reads_real_attributes() {
        let c = parse_cookie("sid=abc; Secure; HttpOnly; SameSite=Strict");
        assert!(c.secure);
        assert!(c.http_only);
        assert_eq!(c.same_site.as_deref(), Some("strict"));
    }
}
