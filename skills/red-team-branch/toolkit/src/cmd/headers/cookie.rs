//! Structural Set-Cookie attribute parsing — split on `;` and match whole
//! attribute tokens so a value substring (`insecure`) cannot forge a flag.
use super::check::CookieCheck;

const ATTR_SPLIT: char = ';';
const SAMESITE_PREFIX: &str = "samesite=";

pub(crate) fn parse_cookie(raw: &str) -> CookieCheck {
    let mut attrs = raw.split(ATTR_SPLIT).map(str::trim);
    let name = attrs
        .next()
        .and_then(|p| p.split_once('='))
        .map(|(k, _)| k.to_string())
        .unwrap_or_default();
    let attrs: Vec<String> = attrs.map(|a| a.to_ascii_lowercase()).collect();
    CookieCheck {
        name,
        secure: attrs.iter().any(|a| a == "secure"),
        http_only: attrs.iter().any(|a| a == "httponly"),
        same_site: parse_samesite(&attrs),
    }
}

fn parse_samesite(attrs: &[String]) -> Option<String> {
    attrs
        .iter()
        .find_map(|a| a.strip_prefix(SAMESITE_PREFIX))
        .map(str::to_string)
}
