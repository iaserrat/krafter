//! Was a symbol marked deprecated at its declaration? Scan the contiguous block
//! of attribute/decorator/comment lines directly above it for a deprecation
//! marker — Rust `#[deprecated]`, JSDoc/Python `@deprecated`, Go `// Deprecated:`.
//! Removing an already-deprecated symbol still breaks callers, but it had a
//! migration window, so the diff reports it under a distinct, lower reason.

/// True if the lines immediately above `start_line` (1-based) carry a
/// deprecation marker. Walks upward over the annotation/comment block only,
/// stopping at the first blank or code line so it can't reach a prior item.
pub fn is_deprecated(text: &str, start_line: usize) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if start_line == 0 {
        return false;
    }
    let mut i = start_line - 1; // index of the line above the declaration
    while i > 0 {
        let line = lines.get(i - 1).map(|s| s.trim()).unwrap_or("");
        if !is_annotation_or_comment(line) {
            break;
        }
        if line.to_ascii_lowercase().contains("deprecated") {
            return true;
        }
        i -= 1;
    }
    false
}

/// An attribute (`#[...]`), decorator (`@...`), or comment line — the only lines
/// that legitimately sit between a declaration and the item above it.
fn is_annotation_or_comment(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    const LEADS: [&str; 6] = ["#[", "#!", "//", "/*", "*", "@"];
    LEADS.iter().any(|p| line.starts_with(p)) || line.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_deprecated_attribute_above_fn() {
        let s = "struct X;\n\n#[deprecated(note = \"use bar\")]\npub fn foo() {}";
        assert!(is_deprecated(s, 4));
    }

    #[test]
    fn plain_function_is_not_deprecated() {
        let s = "struct X;\n\npub fn foo() {}";
        assert!(!is_deprecated(s, 3));
    }

    #[test]
    fn marker_across_blank_line_does_not_leak() {
        // the #[deprecated] belongs to a *prior* item, separated by a blank line.
        let s = "#[deprecated]\npub fn old() {}\n\npub fn current() {}";
        assert!(!is_deprecated(s, 4));
    }
}
