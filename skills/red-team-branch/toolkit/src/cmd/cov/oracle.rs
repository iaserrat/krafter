use super::{bits::*, classify, error, fingerprint, scan, Fnv};
use crate::http::Outcome;

const SERVER_ERROR_MIN: u16 = 500;
const SERVER_ERROR_MAX_EXCLUSIVE: u16 = 600;

const ENTITY_FORMS: &[(u8, &[&[u8]])] = &[
    (b'<', &[b"&lt;", b"&#x3c;", b"&#60;"]),
    (b'>', &[b"&gt;", b"&#x3e;", b"&#62;"]),
    (b'&', &[b"&amp;", b"&#x26;", b"&#38;"]),
    (b'"', &[b"&quot;", b"&#x22;", b"&#34;"]),
    (b'\'', &[b"&#x27;", b"&#39;", b"&apos;"]),
];

pub fn bucket(o: &Outcome, base: &Outcome, canary: &str, payload: &[u8], lat_lower: f64) -> u64 {
    let mut h = Fnv::new();
    h.u64(fingerprint::novelty_key(o, canary, payload));
    h.u8(oracle_mask(o, base, canary, payload, lat_lower) & STABLE_MASK);
    h.finish()
}

pub fn oracle_mask(o: &Outcome, base: &Outcome, _: &str, payload: &[u8], lat_lower: f64) -> u8 {
    if let Some(mask) = transport_mask(o) {
        return mask;
    }
    let mut m = 0;
    m |= ((SERVER_ERROR_MIN..SERVER_ERROR_MAX_EXCLUSIVE).contains(&o.status) as u8) * B_5XX;
    m |= ((o.latency_ms as f64 > lat_lower) as u8) * B_LATENCY;
    m |= (reflected(o, base, payload) as u8) * B_REFLECT;
    m |= encoded_reflect(o, base, payload, m) * B_ENCREFLECT;
    m |= errsig(o, base, payload) * B_ERRSIG;
    m |= diff(o, base) * B_DIFF;
    m
}

fn transport_mask(o: &Outcome) -> Option<u8> {
    let err = o.error.as_ref()?.to_lowercase();
    Some(if err.contains("timeout") || err.contains("timed out") {
        B_TIMEOUT
    } else if ["reset", "refused", "closed", "eof", "broken pipe"]
        .iter()
        .any(|s| err.contains(s))
    {
        B_RESET
    } else {
        0
    })
}

// Raw reflection requires baseline subtraction: present in the response AND
// absent from the baseline body, else a constant body is a false positive.
fn reflected(o: &Outcome, base: &Outcome, payload: &[u8]) -> bool {
    !payload.is_empty()
        && scan::contains(&o.body_raw, payload)
        && !scan::contains(&base.body_raw, payload)
}

fn encoded_reflect(o: &Outcome, base: &Outcome, payload: &[u8], mask: u8) -> u8 {
    if mask & B_REFLECT != 0 || payload.is_empty() {
        return 0;
    }
    let ob = scan::lower_capped(&o.body_raw);
    let bb = scan::lower_capped(&base.body_raw);
    ENTITY_FORMS.iter().any(|&(sp, forms)| {
        payload.contains(&sp)
            && forms
                .iter()
                .any(|f| scan::contains(&ob, f) && !scan::contains(&bb, f))
    }) as u8
}

fn errsig(o: &Outcome, base: &Outcome, payload: &[u8]) -> u8 {
    error::err_family_excl(&o.body_raw, payload)
        .is_some_and(|fam| error::err_family(&base.body_raw) != Some(fam)) as u8
}

fn diff(o: &Outcome, base: &Outcome) -> u8 {
    (crate::http::status_class(o.status) != crate::http::status_class(base.status)
        || classify::len_bucket(o.body_len) != classify::len_bucket(base.body_len)) as u8
}
