//! A union of common keywords across supported languages. Kept verbatim during
//! normalization so structure differentiates clones; other identifiers collapse
//! to a placeholder. A union (vs per-language) is intentionally conservative:
//! it may slightly under-merge Type-2 clones, never over-merge.

const KEYWORDS: &[&str] = &[
    "if", "else", "elif", "for", "while", "loop", "return", "break", "continue", "match", "switch",
    "case", "default", "fn", "func", "def", "function", "class", "struct", "trait", "impl",
    "interface", "enum", "mod", "module", "namespace", "use", "import", "from", "package", "pub",
    "public", "private", "protected", "static", "const", "let", "var", "mut", "new", "self", "this",
    "super", "async", "await", "yield", "try", "catch", "finally", "throw", "throws", "in", "of",
    "is", "as", "and", "or", "not", "true", "false", "null", "nil", "none", "void", "type", "where",
    "do", "then", "end", "begin", "with", "lambda", "go", "defer", "select", "chan",
];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}
