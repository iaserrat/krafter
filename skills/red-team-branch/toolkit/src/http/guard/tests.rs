use super::{guard_target, is_local};

const DENY: bool = false;
const ALLOW: bool = true;

fn accepts(target: &str) -> bool {
    guard_target(target, DENY, &[]).is_ok()
}

#[test]
fn local_hosts_accepted() {
    for host in [
        "localhost",
        "LOCALHOST",
        "127.0.0.1",
        "127.1.2.3",
        "::1",
        "[::1]",
        "0.0.0.0",
    ] {
        assert!(is_local(host), "{host} should be local");
    }
}

#[test]
fn lookalike_public_hosts_rejected() {
    let hosts = [
        "127.0.0.1.evil.com",
        "127.evil.com",
        "evil.com.localhost",
        "localhost.evil.com",
        "evil.com",
        "169.254.169.254",
    ];
    for host in hosts {
        assert!(!is_local(host), "{host} must not be treated as local");
    }
}

// Encoded forms of 127.0.0.1: the `url` crate canonicalizes them to "127.0.0.1"
// BEFORE the guard sees the host, so they resolve to loopback and are accepted.
// This is fail-safe: the target genuinely IS loopback, not an egress bypass.
#[test]
fn encoded_loopback_ips_canonicalize_to_loopback() {
    for target in [
        "http://2130706433/",  // decimal 127.0.0.1
        "http://0x7f000001/",  // hex 127.0.0.1
        "http://0177.0.0.1/",  // octal first octet
    ] {
        assert!(accepts(target), "{target} canonicalizes to loopback");
    }
}

// IPv4-mapped IPv6 loopback is NOT std::net loopback, so the guard rejects it
// without --allow-remote (it is treated as a non-local v6 host).
#[test]
fn ipv4_mapped_loopback_rejected_without_allow_remote() {
    assert!(
        !accepts("http://[::ffff:127.0.0.1]/"),
        "::ffff:127.0.0.1 must not be treated as local"
    );
}

#[test]
fn canonical_loopback_targets_accepted() {
    for target in [
        "http://127.0.0.1/",
        "http://[::1]/",
        "http://localhost/",
        "http://0.0.0.0/",
    ] {
        assert!(accepts(target), "{target} must be accepted");
    }
}

// A non-local lookalike host is refused by default and only opened by the
// explicit --allow-remote escape hatch.
#[test]
fn remote_host_gated_by_allow_remote() {
    assert!(!accepts("http://169.254.169.254/"), "metadata IP refused");
    assert!(
        guard_target("http://169.254.169.254/", ALLOW, &[]).is_ok(),
        "--allow-remote opens the gate"
    );
}
