use super::{fields, hit::Hit};
use crate::{cmd::cov, http};

pub fn classify(
    id: String,
    a: http::Outcome,
    anon: http::Outcome,
    b: Option<http::Outcome>,
) -> Hit {
    let a_ok = is2xx(a.status);
    let anon_ok = is2xx(anon.status);
    let fields = fields::json_keys(&a.body_raw);
    let blocked = cov::block_reason(&a);
    let class = class_name(&a, &anon, b.as_ref(), a_ok, anon_ok);
    Hit {
        id,
        class,
        a_status: a.status,
        anon_status: anon.status,
        b_status: b.as_ref().map(|b| b.status),
        body_len: a.body_len,
        sha8: a.body_sha8,
        keys: fields.keys,
        sensitive: fields.sensitive,
        snippet: if a_ok { a.snippet } else { String::new() },
        blocked,
    }
}

fn class_name(
    a: &http::Outcome,
    anon: &http::Outcome,
    b: Option<&http::Outcome>,
    a_ok: bool,
    anon_ok: bool,
) -> &'static str {
    if !a_ok {
        "denied"
    } else if anon_ok && anon.body_sha8 == a.body_sha8 {
        "public"
    } else if b.is_some_and(|b| is2xx(b.status) && b.body_sha8 == a.body_sha8 && !anon_ok) {
        "cross_user_proven"
    } else if b.is_some_and(|b| !is2xx(b.status)) {
        // Negative control: compare actor was DENIED this record, so A reading
        // it is correct per-user scoping, not cross-user access.
        "scoped"
    } else {
        "accessible"
    }
}

fn is2xx(status: u16) -> bool {
    http::is_success(status)
}
