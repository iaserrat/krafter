//! TS/JS class, interface, and enum members. A class member is public when its
//! class is exported and it is not `private`/`protected`/`#`-private; interface
//! and enum members are public when their container is exported.

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children, text};
use crate::surface::rules::common::{qual, symbol, vis, WHOLE};
use tree_sitter::Node;

pub fn class(n: Node, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    out.push(symbol("Class", name.to_string(), n, "body", vis(exported), src));
    for m in body_members(n) {
        match m.kind() {
            "method_definition" => push_member(m, "Method", "body", name, exported, src, out),
            "public_field_definition" | "field_definition" => {
                push_member(m, "Field", WHOLE, name, exported, src, out)
            }
            _ => {}
        }
    }
}

fn push_member(m: Node, kind: &str, body: &str, cls: &str, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    if let Some(name) = field_text(m, src, "name") {
        let public = exported && !restricted(m, src) && !name.starts_with('#');
        out.push(symbol(kind, qual(cls, name), m, body, vis(public), src));
    }
}

/// A `private`/`protected` accessibility modifier on a class member.
fn restricted(m: Node, src: &str) -> bool {
    named_children(m).iter().any(|c| {
        c.kind() == "accessibility_modifier" && matches!(text(*c, src).trim(), "private" | "protected")
    })
}

pub fn interface(n: Node, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    out.push(symbol("Interface", name.to_string(), n, "body", vis(exported), src));
    for m in body_members(n) {
        if matches!(m.kind(), "property_signature" | "method_signature") {
            if let Some(mname) = field_text(m, src, "name") {
                out.push(symbol("Member", qual(name, mname), m, WHOLE, vis(exported), src));
            }
        }
    }
}

pub fn enumeration(n: Node, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(n, src, "name").unwrap_or_default();
    out.push(symbol("Enum", name.to_string(), n, "body", vis(exported), src));
    for m in body_members(n) {
        let vname = field_text(m, src, "name").or(if m.kind() == "property_identifier" {
            Some(text(m, src))
        } else {
            None
        });
        if let Some(vn) = vname {
            out.push(symbol("Variant", qual(name, vn), m, WHOLE, vis(exported), src));
        }
    }
}

/// Named children of the container's `body`.
fn body_members(n: Node) -> Vec<Node> {
    n.child_by_field_name("body").map(named_children).unwrap_or_default()
}
