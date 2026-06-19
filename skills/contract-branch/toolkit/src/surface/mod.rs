//! Public-contract surface extraction: parse the file with tree-sitter, then run
//! the language's contract rules over the tree to produce the deterministic set
//! of public symbols (functions, methods, types, fields, variants, constants)
//! the `assess` command diffs.

pub mod deprecate;
pub mod lang;
pub mod model;
pub mod node;
pub mod parse;
pub mod rules;
pub mod sig;

pub use model::{FileSurface, Symbol};

use lang::Lang;
use std::path::Path;

/// Extract the contract surface of one file's bytes. Never panics: an unknown
/// extension or a grammar that fails to load yields the honesty flags
/// (`parse_ok` / `vis_supported`) with an empty symbol list. Syntax errors still
/// yield whatever parsed, with `parse_ok=false`.
pub fn extract(path: &Path, bytes: Vec<u8>) -> FileSurface {
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let display = path.display().to_string();
    let Some(lang) = Lang::from_path(path) else {
        return unmeasured(display);
    };
    let Some(tree) = parse::parse(lang, &text) else {
        return unmeasured(display);
    };
    let root = tree.root_node();
    let mut symbols = rules::symbols(lang, root, &text);
    symbols.sort_by(|a, b| a.start_line.cmp(&b.start_line).then_with(|| a.name.cmp(&b.name)));
    FileSurface {
        path: display,
        lang: lang.name().to_string(),
        parse_ok: !root.has_error(),
        vis_supported: true,
        symbols,
    }
}

fn unmeasured(path: String) -> FileSurface {
    FileSurface {
        path,
        lang: "unsupported".to_string(),
        parse_ok: false,
        vis_supported: false,
        symbols: Vec::new(),
    }
}
