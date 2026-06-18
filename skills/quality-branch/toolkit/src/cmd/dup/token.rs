//! Deterministic tokenizer + normalization. Strips comments/whitespace; emits
//! one hashed token each. Type-2 (default) collapses identifiers/strings/numbers
//! to placeholders so renamed clones match; keywords and punctuation stay
//! verbatim because they carry the structure that prevents false positives.

use crate::cmd::dup::fingerprint::fnv1a;
use crate::cmd::dup::keywords::is_keyword;
use crate::cmd::dup::lex;

#[derive(Clone, Copy)]
pub enum CommentStyle {
    Slash,
    Hash,
}

pub struct Token {
    pub hash: u64,
    pub line: usize,
}

const ID: &[u8] = b"$ID";
const STR: &[u8] = b"$STR";
const NUM: &[u8] = b"$NUM";

pub fn tokenize(bytes: &[u8], style: CommentStyle, type2: bool) -> Vec<Token> {
    let mut out = Vec::new();
    let mut line = 1usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if let Some(j) = lex::skip_comment(bytes, i, style, &mut line) {
            i = j;
            continue;
        }
        let start = line;
        if c == b'"' || c == b'\'' {
            let j = lex::skip_string(bytes, i, &mut line);
            let h = if type2 { fnv1a(STR) } else { fnv1a(&bytes[i..j]) };
            out.push(Token { hash: h, line: start });
            i = j;
        } else if c.is_ascii_digit() {
            let j = lex::read_number(bytes, i);
            let h = if type2 { fnv1a(NUM) } else { fnv1a(&bytes[i..j]) };
            out.push(Token { hash: h, line: start });
            i = j;
        } else if c == b'_' || c.is_ascii_alphabetic() {
            let j = lex::read_ident(bytes, i);
            let word = &bytes[i..j];
            let kw = std::str::from_utf8(word).map(is_keyword).unwrap_or(false);
            let h = if kw || !type2 { fnv1a(word) } else { fnv1a(ID) };
            out.push(Token { hash: h, line: start });
            i = j;
        } else {
            out.push(Token {
                hash: fnv1a(&bytes[i..i + 1]),
                line: start,
            });
            i += 1;
        }
    }
    out
}
