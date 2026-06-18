use super::{classify, error, scan, Fnv};
use crate::http::Outcome;

const FP_CAP: usize = 4096;
const ENTITY_TOKENS: &[&[u8]] = &[
    b"&lt;", b"&gt;", b"&amp;", b"&#x", b"&#", b"&quot;", b"&#39;",
];

pub fn body_fp(o: &Outcome, canary: &str, payload: &[u8]) -> u64 {
    let mut body = o.body_raw[..o.body_raw.len().min(FP_CAP)].to_vec();
    body.make_ascii_lowercase();
    scan::blank(&mut body, canary.as_bytes());
    if !payload.is_empty() {
        scan::blank(&mut body, &payload.to_ascii_lowercase());
    }
    for token in ENTITY_TOKENS {
        scan::blank(&mut body, token);
    }
    hash_normalized(&body)
}

pub fn novelty_key(o: &Outcome, canary: &str, payload: &[u8]) -> u64 {
    let mut h = Fnv::new();
    h.u8(if o.error.is_some() {
        0
    } else {
        crate::http::status_class(o.status) as u8
    });
    h.u8(error::err_family_excl(&o.body_raw, payload)
        .map(|i| (i + 1) as u8)
        .unwrap_or(0));
    h.u8(classify::len_bucket(o.body_len));
    h.str(classify::ctype_class(o));
    h.u64(body_fp(o, canary, payload));
    h.finish()
}

fn hash_normalized(body: &[u8]) -> u64 {
    let mut h = Fnv::new();
    let (mut last_ws, mut last_hex) = (false, false);
    for &c in body {
        let normalized = normalized_byte(c, last_ws, last_hex);
        if let Some(byte) = normalized.emit {
            h.u8(byte);
        }
        last_ws = normalized.last_ws;
        last_hex = normalized.last_hex;
    }
    h.finish()
}

struct NormalizedByte {
    emit: Option<u8>,
    last_ws: bool,
    last_hex: bool,
}

fn normalized_byte(c: u8, last_ws: bool, last_hex: bool) -> NormalizedByte {
    if matches!(c, b' ' | b'\t' | b'\n' | b'\r') {
        NormalizedByte {
            emit: if last_ws { None } else { Some(b' ') },
            last_ws: true,
            last_hex: false,
        }
    } else if c.is_ascii_hexdigit() {
        NormalizedByte {
            emit: if last_hex { None } else { Some(b'#') },
            last_ws: false,
            last_hex: true,
        }
    } else {
        NormalizedByte {
            emit: Some(c),
            last_ws: false,
            last_hex: false,
        }
    }
}
