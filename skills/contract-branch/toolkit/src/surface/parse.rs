//! Parse source into a tree-sitter tree. Deterministic: the same bytes always
//! yield the same tree, so the extracted contract is reproducible.

use crate::surface::lang::Lang;
use tree_sitter::{Parser, Tree};

/// Parse `src` as `lang`. Returns `None` only if the grammar fails to load
/// (a build/ABI problem), never on syntax errors — a partial tree still yields
/// whatever declarations parsed, and the caller reads `tree.root_node().has_error()`.
pub fn parse(lang: Lang, src: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&lang.grammar()).ok()?;
    parser.parse(src, None)
}
