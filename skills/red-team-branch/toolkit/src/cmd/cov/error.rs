use super::scan::{blank, contains, lower_capped};
use crate::http::Outcome;

pub struct ErrorSignature {
    pub family: &'static str,
    pub patterns: &'static [&'static str],
}

pub const SIGS: &[ErrorSignature] = &[
    ErrorSignature {
        family: "deser",
        patterns: &[
            "invalidclassexception",
            "classnotfoundexception",
            "objectinputstream",
            "unserialize()",
            "yaml.load",
        ],
    },
    ErrorSignature {
        family: "sql",
        patterns: &[
            "you have an error in your sql syntax",
            "unclosed quotation mark after the character string",
            "quoted string not properly terminated",
            "warning: mysqli",
            "pg_query",
            "syntax error at or near",
            "sqlite error",
            "db2 sql error",
            "[sql server]",
            "sqlstate",
        ],
    },
    ErrorSignature {
        family: "stack",
        patterns: &[
            "traceback (most recent call last)",
            "java.lang.",
            "goroutine ",
            "panic:",
            "fatal error:",
            "stack trace:",
        ],
    },
    ErrorSignature {
        family: "path",
        patterns: &["/var/www/", "root:x:0:0", "/etc/passwd"],
    },
];

pub fn err_family(body: &[u8]) -> Option<usize> {
    err_family_excl(body, b"")
}

pub fn err_family_excl(body: &[u8], exclude: &[u8]) -> Option<usize> {
    let mut lower = lower_capped(body);
    if !exclude.is_empty() {
        blank(&mut lower, &exclude.to_ascii_lowercase());
    }
    SIGS.iter()
        .enumerate()
        .find(|(_, sig)| sig.patterns.iter().any(|p| contains(&lower, p.as_bytes())))
        .map(|(i, _)| i)
}

pub fn err_family_id(o: &Outcome) -> u8 {
    err_family(&o.body_raw).map(|i| (i + 1) as u8).unwrap_or(0)
}

pub fn family_name(id: u8) -> &'static str {
    if id == 0 {
        ""
    } else {
        SIGS.get((id - 1) as usize)
            .map(|sig| sig.family)
            .unwrap_or("")
    }
}
