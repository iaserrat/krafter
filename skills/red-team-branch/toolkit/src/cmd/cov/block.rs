use super::scan::{contains, lower_capped};
use crate::http::Outcome;

const RATE_LIMITED_STATUS: u16 = 429;
const SERVICE_UNAVAILABLE_STATUS: u16 = 503;
const WAF_STATUS_CODES: &[u16] = &[401, 403, 406];

const WAF_SIGS: &[&str] = &[
    "cloudflare",
    "access denied",
    "request blocked",
    "attention required",
    "akamai",
    "incapsula",
    "mod_security",
    "web application firewall",
    "captcha",
];

pub fn block_reason(o: &Outcome) -> Option<String> {
    if o.error.is_some() {
        return None;
    }
    match o.status {
        RATE_LIMITED_STATUS => Some("HTTP 429 (rate-limited)".into()),
        SERVICE_UNAVAILABLE_STATUS => Some("HTTP 503 (service unavailable)".into()),
        status if WAF_STATUS_CODES.contains(&status) && has_waf_sig(o) => {
            Some(format!("WAF block signature (HTTP {})", o.status))
        }
        _ => None,
    }
}

fn has_waf_sig(o: &Outcome) -> bool {
    let lower = lower_capped(&o.body_raw);
    WAF_SIGS.iter().any(|w| contains(&lower, w.as_bytes()))
}
