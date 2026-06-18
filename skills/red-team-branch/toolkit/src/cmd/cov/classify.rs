use crate::http::Outcome;

const LEN_4_TO_7: std::ops::RangeInclusive<usize> = 4..=7;
const LEN_8_TO_15: std::ops::RangeInclusive<usize> = 8..=15;
const LEN_16_TO_31: std::ops::RangeInclusive<usize> = 16..=31;
const LEN_32_TO_127: std::ops::RangeInclusive<usize> = 32..=127;
const LEN_128_TO_2047: std::ops::RangeInclusive<usize> = 128..=2047;
const LEN_2048_TO_65535: std::ops::RangeInclusive<usize> = 2048..=65535;
const TEXT_SNIFF_BYTES: usize = 512;

pub fn len_bucket(n: usize) -> u8 {
    match n {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        n if LEN_4_TO_7.contains(&n) => 4,
        n if LEN_8_TO_15.contains(&n) => 5,
        n if LEN_16_TO_31.contains(&n) => 6,
        n if LEN_32_TO_127.contains(&n) => 7,
        n if LEN_128_TO_2047.contains(&n) => 8,
        n if LEN_2048_TO_65535.contains(&n) => 9,
        _ => 10,
    }
}

pub fn ctype_class(o: &Outcome) -> &'static str {
    if let Some(class) = declared_type(o) {
        return class;
    }
    sniff_body(&o.body_raw)
}

fn declared_type(o: &Outcome) -> Option<&'static str> {
    let ct = o.content_type.as_ref()?.to_ascii_lowercase();
    ["json", "html", "xml"]
        .into_iter()
        .find(|kind| ct.contains(kind))
        .or_else(|| ct.contains("text/").then_some("text"))
}

fn sniff_body(body: &[u8]) -> &'static str {
    let body = body.trim_ascii_start();
    if body.is_empty() {
        ""
    } else if matches!(body.first(), Some(b'{' | b'[')) {
        "json"
    } else if starts_ci(body, b"<!doctype") || starts_ci(body, b"<html") {
        "html"
    } else if starts_ci(body, b"<?xml") {
        "xml"
    } else if body.iter().take(TEXT_SNIFF_BYTES).all(|&c| is_text_byte(c)) {
        "text"
    } else {
        "bin"
    }
}

fn starts_ci(body: &[u8], prefix: &[u8]) -> bool {
    body.len() >= prefix.len() && body[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn is_text_byte(c: u8) -> bool {
    matches!(c, b'\t' | b'\n' | b'\r' | b' '..=b'~')
}
