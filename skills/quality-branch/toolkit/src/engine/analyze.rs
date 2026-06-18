//! Entry point: source bytes + a path (for language detection by extension) ->
//! a FileAnalysis. Bytes may come from the working tree or a git blob, so the
//! same code measures both sides of a delta with no temp files.

use crate::engine::flatten::flatten;
use crate::engine::model::FileAnalysis;
use rust_code_analysis::{get_function_spaces, get_language_for_file};
use std::path::Path;

const UNSUPPORTED: &str = "unsupported";

/// Analyze one file's bytes. Never panics: unknown language or a failed parse
/// returns `parse_ok=false` with an empty function list.
pub fn analyze(path: &Path, source: Vec<u8>) -> FileAnalysis {
    let display = path.display().to_string();
    let Some(lang) = get_language_for_file(path) else {
        return unparsed(display, UNSUPPORTED.to_string());
    };
    let lang_name = format!("{lang:?}");
    match get_function_spaces(&lang, source, path, None) {
        Some(space) => FileAnalysis {
            path: display,
            lang: lang_name,
            parse_ok: true,
            functions: flatten(&space),
        },
        None => unparsed(display, lang_name),
    }
}

fn unparsed(path: String, lang: String) -> FileAnalysis {
    FileAnalysis {
        path,
        lang,
        parse_ok: false,
        functions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_branches_raise_cognitive() {
        let src = b"fn f(a: i32) -> i32 { if a > 0 { if a > 1 { for _ in 0..a { return a; } } } 0 }".to_vec();
        let fa = analyze(Path::new("f.rs"), src);
        assert!(fa.parse_ok);
        let f = fa.functions.iter().find(|m| m.name.contains('f')).unwrap();
        assert!(f.cognitive > 0.0, "nested control flow should score cognitive");
    }

    #[test]
    fn unknown_extension_is_not_ok() {
        let fa = analyze(Path::new("data.bin"), b"\x00\x01".to_vec());
        assert!(!fa.parse_ok);
        assert_eq!(fa.lang, UNSUPPORTED);
    }
}
