//! Map a file path to the language whose contract ctk knows how to read, and to
//! the tree-sitter grammar that parses it. One uniform parse mechanism; the
//! per-language *rules* live in `rules/`. A path with no known extension yields
//! `None` and is reported unmeasured.

use tree_sitter::Language;

/// A language ctk extracts a public contract from. `Tsx` also parses plain JS
/// and JSX (the TSX grammar is a superset); `Ts` parses `.ts`/`.mts`/`.cts`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Ts,
    Tsx,
    Python,
    Go,
}

impl Lang {
    /// Detect from the file extension, or `None` if unsupported.
    pub fn from_path(path: &std::path::Path) -> Option<Lang> {
        match path.extension()?.to_str()? {
            "rs" => Some(Lang::Rust),
            "ts" | "mts" | "cts" => Some(Lang::Ts),
            "tsx" | "js" | "jsx" | "mjs" | "cjs" => Some(Lang::Tsx),
            "py" | "pyi" => Some(Lang::Python),
            "go" => Some(Lang::Go),
            _ => None,
        }
    }

    /// The tree-sitter grammar for this language.
    pub fn grammar(&self) -> Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::Ts => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
        }
    }

    /// Reporting name for the `lang` field.
    pub fn name(&self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Ts => "typescript",
            Lang::Tsx => "tsx",
            Lang::Python => "python",
            Lang::Go => "go",
        }
    }
}
