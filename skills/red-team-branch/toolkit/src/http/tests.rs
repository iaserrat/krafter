use super::send::raw_header;

// Adversarial control for the header-injection guard: a raw header value with
// any CR/LF/NUL/control byte must be refused so the fuzz Header injector cannot
// smuggle a second header or request line. A benign value must pass through.

#[test]
fn raw_header_rejects_crlf_and_nul_injection() {
    for evil in [
        &b"value\r\nX-Injected: 1"[..],
        &b"value\rmore"[..],
        &b"value\nmore"[..],
        &b"value\0nul"[..],
    ] {
        assert!(
            raw_header(evil).is_err(),
            "must reject injection bytes in {evil:?}"
        );
    }
}

#[test]
fn raw_header_accepts_benign_value() {
    let value = raw_header(b"Bearer s3cr3t-token.value").expect("benign value accepted");
    assert_eq!(value.to_str().ok(), Some("Bearer s3cr3t-token.value"));
}
