use super::cookie::parse_cookie;
use crate::{cmd, config::Ctx, http};
use std::collections::BTreeMap;

const SET_COOKIE: &str = "set-cookie";

pub struct HeaderReport {
    pub headers: BTreeMap<String, Vec<String>>,
    pub set_cookie: Vec<CookieCheck>,
    pub issues: Vec<String>,
    pub error: Option<String>,
}

pub struct CookieCheck {
    pub name: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

pub async fn fetch_headers(client: &reqwest::Client, ctx: &Ctx, url: &str, path: &str, extra: &[(String, String)]) -> anyhow::Result<HeaderReport> {
    let header_strs: Vec<String> = extra.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    let full_url = format!("{}{}", url.trim_end_matches('/'), path);
    let spec = cmd::base_spec(ctx, "GET", &full_url, &header_strs, None)?;
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    // Transport failure: an empty header map is not evidence of missing controls.
    if let Some(error) = out.error {
        return Ok(HeaderReport { headers: BTreeMap::new(), set_cookie: Vec::new(), issues: Vec::new(), error: Some(error) });
    }
    let mut headers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut cookies = Vec::new();
    let mut issues = Vec::new();
    for (name, value) in &out.headers {
        let key = name.to_ascii_lowercase();
        if key == SET_COOKIE {
            cookies.push(parse_cookie(value));
        }
        headers.entry(key).or_default().push(value.clone());
    }
    check_security_headers(&headers, &mut issues);
    check_cookies(&cookies, &mut issues);
    Ok(HeaderReport { headers, set_cookie: cookies, issues, error: None })
}

pub(crate) fn check_security_headers(headers: &BTreeMap<String, Vec<String>>, issues: &mut Vec<String>) {
    if !headers.contains_key("strict-transport-security") {
        issues.push("missing-hsts".into());
    }
    if !headers.contains_key("x-frame-options") && !headers.contains_key("content-security-policy") {
        issues.push("missing-frame-protection".into());
    }
    if !headers.contains_key("x-content-type-options") {
        issues.push("missing-x-content-type-options".into());
    }
    if !headers.contains_key("content-security-policy") {
        issues.push("missing-csp".into());
    }
    if headers.contains_key("server") {
        issues.push("server-header-disclosed".into());
    }
    if let Some(csp) = headers.get("content-security-policy").and_then(|v| v.first()) {
        if csp.contains("unsafe-inline") { issues.push("csp-allows-unsafe-inline".into()); }
        if csp.contains("unsafe-eval") { issues.push("csp-allows-unsafe-eval".into()); }
    }
}

pub(crate) fn check_cookies(cookies: &[CookieCheck], issues: &mut Vec<String>) {
    for c in cookies {
        if !c.secure { issues.push(format!("cookie-{}-missing-secure", c.name)); }
        if !c.http_only { issues.push(format!("cookie-{}-missing-httponly", c.name)); }
        if c.same_site.is_none() {
            issues.push(format!("cookie-{}-missing-samesite", c.name));
        }
    }
}
