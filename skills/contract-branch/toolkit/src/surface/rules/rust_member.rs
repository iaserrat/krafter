//! Rust type members: struct fields, enum variants, trait methods, and inherent
//! `impl` methods. A member is public only if its container is public too.

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children};
use crate::surface::rules::common::{qual, symbol, vis, WHOLE};
use crate::surface::rules::rust::is_pub;
use tree_sitter::Node;

pub fn structure(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    let public = is_pub(n, src);
    out.push(symbol("Struct", qual(prefix, name), n, "body", vis(public), src));
    let ty = qual(prefix, name);
    for f in body_children(n, "field_declaration") {
        if let Some(fname) = field_text(f, src, "name") {
            out.push(symbol("Field", qual(&ty, fname), f, WHOLE, vis(public && is_pub(f, src)), src));
        }
    }
}

pub fn enumeration(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    let public = is_pub(n, src);
    out.push(symbol("Enum", qual(prefix, name), n, "body", vis(public), src));
    let ty = qual(prefix, name);
    for v in body_children(n, "enum_variant") {
        if let Some(vname) = field_text(v, src, "name") {
            out.push(symbol("Variant", qual(&ty, vname), v, WHOLE, vis(public), src));
        }
    }
}

pub fn trait_def(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    let public = is_pub(n, src);
    out.push(symbol("Trait", qual(prefix, name), n, "body", vis(public), src));
    let ty = qual(prefix, name);
    for m in trait_methods(n) {
        let mname = field_text(m, src, "name").unwrap_or_default();
        out.push(symbol("Method", qual(&ty, mname), m, "body", vis(public), src));
    }
}

/// Trait items can be provided (`function_item`) or required
/// (`function_signature_item`, no body); both are part of the contract.
fn trait_methods(n: Node) -> Vec<Node> {
    n.child_by_field_name("body")
        .map(|b| {
            named_children(b)
                .into_iter()
                .filter(|c| matches!(c.kind(), "function_item" | "function_signature_item"))
                .collect()
        })
        .unwrap_or_default()
}

pub fn impl_block(n: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    if n.child_by_field_name("trait").is_some() {
        return; // a trait impl realizes the trait's contract, it does not define one
    }
    let ty = field_text(n, src, "type").unwrap_or_default();
    let base = qual(prefix, ty.split('<').next().unwrap_or(ty).trim());
    for m in body_children(n, "function_item") {
        if is_pub(m, src) {
            let mname = field_text(m, src, "name").unwrap_or_default();
            out.push(symbol("Method", qual(&base, mname), m, "body", vis(true), src));
        }
    }
}

/// Named children of `n`'s `body` with kind `kind`.
fn body_children<'a>(n: Node<'a>, kind: &str) -> Vec<Node<'a>> {
    n.child_by_field_name("body")
        .map(|b| named_children(b).into_iter().filter(|c| c.kind() == kind).collect())
        .unwrap_or_default()
}
