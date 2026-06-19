//! Signature shaping. A declaration's signature comes from its parse-tree node
//! text; here we make it both readable (display) and formatter-proof (compare).

/// Human-readable form: collapse whitespace, strip a trailing `{`/`=`/`,`/`;`
/// left over from cutting a header off its body or value.
pub fn normalize(s: &str) -> String {
    let joined = s.split_whitespace().collect::<Vec<_>>().join(" ");
    joined.trim_end_matches(['{', '=', ',', ';', ':']).trim().to_string()
}

/// Whitespace-insensitive comparison key. A formatter must never read as a
/// contract change: spacing around punctuation (`a:i32` vs `a: i32`) and a
/// trailing comma in a wrapped list are both canonicalized away. Only a real
/// token change (a param, a type, the return) survives.
pub fn canonical(sig: &str) -> String {
    let squeezed: String = sig.chars().filter(|c| !c.is_whitespace()).collect();
    squeezed.replace(",)", ")").replace(",]", "]").replace(",>", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_and_strips_trailing() {
        assert_eq!(normalize("pub fn f(a: i32)  -> i32 {"), "pub fn f(a: i32) -> i32");
        assert_eq!(normalize("pub const LIMIT: usize ="), "pub const LIMIT: usize");
    }

    #[test]
    fn canonical_ignores_formatter_spacing_and_trailing_comma() {
        assert_eq!(
            canonical("pub fn f(a:i32,b:i32)->i32"),
            canonical("pub fn f(a: i32, b: i32) -> i32")
        );
        assert_eq!(canonical("fn f(a: i32,)"), canonical("fn f(a: i32)"));
        assert_ne!(canonical("fn f(a: i32)"), canonical("fn f(a: i32, b: i32)"));
    }
}
