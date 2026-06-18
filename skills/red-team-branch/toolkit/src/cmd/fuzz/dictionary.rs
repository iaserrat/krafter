use std::collections::HashSet;

pub const MAX_DICT: usize = 256;
pub const HARVEST_CAP: usize = 8192;
pub const MIN_TOKEN: usize = 3;
pub const MAX_TOKEN: usize = 32;

pub struct Dictionary {
    pub tokens: Vec<Vec<u8>>,
    pub seen: HashSet<Vec<u8>>,
}

pub fn initial(corpus: &[crate::cmd::mut_engine::Seed]) -> Dictionary {
    let mut dict = Vec::new();
    let mut set = HashSet::new();
    for token in BUILTIN {
        add(&mut dict, &mut set, token.as_bytes().to_vec());
    }
    for seed in corpus {
        harvest(&mut dict, &mut set, &seed.buf);
    }
    Dictionary {
        tokens: dict,
        seen: set,
    }
}

pub fn harvest(dict: &mut Vec<Vec<u8>>, set: &mut HashSet<Vec<u8>>, body: &[u8]) {
    if dict.len() >= MAX_DICT {
        return;
    }
    let mut cur = Vec::new();
    for &byte in body.iter().take(HARVEST_CAP) {
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            cur.push(byte);
        } else {
            try_add(dict, set, &cur);
            cur.clear();
        }
        if dict.len() >= MAX_DICT {
            return;
        }
    }
    try_add(dict, set, &cur);
}

fn add(dict: &mut Vec<Vec<u8>>, set: &mut HashSet<Vec<u8>>, token: Vec<u8>) {
    if !token.is_empty() && dict.len() < MAX_DICT && set.insert(token.clone()) {
        dict.push(token);
    }
}

fn try_add(dict: &mut Vec<Vec<u8>>, set: &mut HashSet<Vec<u8>>, token: &[u8]) {
    if (MIN_TOKEN..=MAX_TOKEN).contains(&token.len()) {
        add(dict, set, token.to_vec());
    }
}

const BUILTIN: &[&str] = &[
    "'",
    "\"",
    "`",
    "</",
    "<script>",
    "../",
    "..\\",
    "%00",
    "${",
    "{{",
    "||",
    "&&",
    "OR",
    "AND",
    "SELECT",
    "UNION",
    "SLEEP",
    "true",
    "false",
    "null",
    "-1",
    "0",
    "127.0.0.1",
    "\r\n",
    "\n",
    "{}",
    "[]",
    ":",
    ";",
    "=",
    "&",
    "<!--",
    "-->",
];
