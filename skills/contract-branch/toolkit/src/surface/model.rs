//! The public-contract surface of a file: the named symbols an external caller
//! could depend on, each with the deterministic facts `assess` diffs across
//! refs — the normalized signature and the visibility.

use serde::Serialize;

/// Whether a symbol is part of the public contract. Reducing `Public` to
/// `NonPublic` between two refs is itself a breaking change.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    NonPublic,
}

/// One named function/method and the contract facts about it.
#[derive(Serialize, Clone)]
pub struct Symbol {
    /// Qualified name (e.g. `Impl::method`), so the same symbol pairs across refs.
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
    /// Whitespace-normalized declaration (params, return type), body stripped.
    pub signature: String,
    pub visibility: Visibility,
    /// Marked `#[deprecated]`/`@deprecated`/`@Deprecated` at its declaration.
    pub deprecated: bool,
}

impl Symbol {
    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }
}

/// One file's contract surface. `vis_supported=false` is the honesty flag: the
/// engine parsed the file but ctk has no visibility rule for its language, so
/// the caller must report it unmeasured rather than infer an empty contract.
#[derive(Serialize, Clone)]
pub struct FileSurface {
    pub path: String,
    pub lang: String,
    pub parse_ok: bool,
    pub vis_supported: bool,
    pub symbols: Vec<Symbol>,
}
