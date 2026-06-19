//! Rust contract rules. Public = a bare `pub` (not `pub(crate)`/`pub(super)`).
//! Walks items, descending `pub mod`s; struct/enum/trait/impl members are in
//! `rust_member`.

use crate::surface::model::Symbol;
use crate::surface::node::{self, field_text, named_children};
use crate::surface::rules::common::{qual, symbol, vis, WHOLE};
use crate::surface::rules::rust_member as member;
use tree_sitter::Node;

pub fn symbols(root: Node, src: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    walk(root, "", src, &mut out);
    out
}

pub(super) fn walk(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    for child in named_children(n) {
        match child.kind() {
            "function_item" | "function_signature_item" => out.push(func(child, prefix, src)),
            "const_item" | "static_item" => out.push(named(child, "Const", "value", prefix, src)),
            "type_item" => out.push(named(child, "TypeAlias", WHOLE, prefix, src)),
            "struct_item" | "union_item" => member::structure(child, prefix, src, out),
            "enum_item" => member::enumeration(child, prefix, src, out),
            "trait_item" => member::trait_def(child, prefix, src, out),
            "impl_item" => member::impl_block(child, prefix, src, out),
            "mod_item" if is_pub(child, src) => descend_mod(child, prefix, src, out),
            _ => {}
        }
    }
}

fn func(n: Node, prefix: &str, src: &str) -> Symbol {
    let name = field_text(n, src, "name").unwrap_or_default();
    let kind = if prefix.is_empty() { "Function" } else { "Method" };
    symbol(kind, qual(prefix, name), n, "body", vis(is_pub(n, src)), src)
}

fn named(n: Node, kind: &str, body_field: &str, prefix: &str, src: &str) -> Symbol {
    let name = field_text(n, src, "name").unwrap_or_default();
    symbol(kind, qual(prefix, name), n, body_field, vis(is_pub(n, src)), src)
}

fn descend_mod(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    if let Some(body) = n.child_by_field_name("body") {
        let name = field_text(n, src, "name").unwrap_or_default();
        walk(body, &qual(prefix, name), src, out);
    }
}

/// A bare `pub` visibility modifier — `pub(crate)`/`pub(super)` are narrower and
/// not part of the crate's public contract.
pub(super) fn is_pub(n: Node, src: &str) -> bool {
    named_children(n)
        .iter()
        .any(|c| c.kind() == "visibility_modifier" && node::text(*c, src).trim() == "pub")
}
