//! Shared helpers for the per-language rule walks: build a `Symbol` from a
//! declaration node, qualify member names, and the common visibility shorthands.

use crate::surface::model::{Symbol, Visibility};
use crate::surface::{deprecate, node, sig};
use tree_sitter::Node;

/// Field name that no node has — passing it to `header` yields the whole node
/// text (for declarations with no body to cut off, e.g. a const or alias).
pub const WHOLE: &str = "";

/// Build a symbol from declaration node `n`. `body_field` is the field whose
/// start ends the signature; `WHOLE` keeps the entire node.
pub fn symbol(kind: &str, name: String, n: Node, body_field: &str, vis: Visibility, src: &str) -> Symbol {
    let line = n.start_position().row + 1;
    Symbol {
        name,
        kind: kind.to_string(),
        start_line: line,
        end_line: n.end_position().row + 1,
        signature: sig::normalize(&node::header(n, src, body_field)),
        visibility: vis,
        deprecated: deprecate::is_deprecated(src, line),
    }
}

/// Qualify a member name under its container (`Type::member`).
pub fn qual(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}::{name}")
    }
}

pub fn vis(public: bool) -> Visibility {
    if public {
        Visibility::Public
    } else {
        Visibility::NonPublic
    }
}

/// Go / capitalization-export rule: the first character is uppercase.
pub fn capitalized(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}
