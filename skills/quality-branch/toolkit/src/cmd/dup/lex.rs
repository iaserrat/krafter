//! Lexing primitives: skip comments/strings, read identifier/number runs. Each
//! advances `line` for any newline it consumes so token line numbers stay exact.

use crate::cmd::dup::token::CommentStyle;

/// Skip a comment at `i` if one starts there; return the index past it.
pub fn skip_comment(b: &[u8], i: usize, style: CommentStyle, line: &mut usize) -> Option<usize> {
    match style {
        CommentStyle::Hash if b[i] == b'#' => Some(to_eol(b, i)),
        CommentStyle::Slash if starts(b, i, b"//") => Some(to_eol(b, i)),
        CommentStyle::Slash if starts(b, i, b"/*") => Some(block(b, i, line)),
        _ => None,
    }
}

fn to_eol(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }
    i
}

fn block(b: &[u8], mut i: usize, line: &mut usize) -> usize {
    i += 2;
    while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
        if b[i] == b'\n' {
            *line += 1;
        }
        i += 1;
    }
    (i + 2).min(b.len())
}

fn starts(b: &[u8], i: usize, p: &[u8]) -> bool {
    b.len() >= i + p.len() && &b[i..i + p.len()] == p
}

/// Skip a quoted string (the opening quote is at `i`), handling backslash escapes.
pub fn skip_string(b: &[u8], i: usize, line: &mut usize) -> usize {
    let quote = b[i];
    let mut j = i + 1;
    while j < b.len() {
        let c = b[j];
        if c == b'\\' {
            if b.get(j + 1) == Some(&b'\n') {
                *line += 1; // escaped (line-continuation) newline still advances the line
            }
            j += 2;
            continue;
        }
        if c == b'\n' {
            *line += 1;
        }
        j += 1;
        if c == quote {
            break;
        }
    }
    j.min(b.len())
}

pub fn read_ident(b: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < b.len() && (b[j] == b'_' || b[j].is_ascii_alphanumeric()) {
        j += 1;
    }
    j
}

pub fn read_number(b: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < b.len() && (b[j] == b'.' || b[j] == b'_' || b[j].is_ascii_alphanumeric()) {
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::skip_string;

    #[test]
    fn escaped_newline_advances_line() {
        let s = b"\"ab\\\ncd\""; // "ab\<newline>cd" — a line-continuation
        let mut line = 1;
        skip_string(s, 0, &mut line);
        assert_eq!(line, 2);
    }

    #[test]
    fn plain_newline_in_string_advances_line() {
        let s = b"\"a\nb\"";
        let mut line = 1;
        skip_string(s, 0, &mut line);
        assert_eq!(line, 2);
    }
}

