//! Python contract rules. Public = a name with no leading underscore (the
//! language convention). Captures module functions, classes and their methods,
//! and annotated class attributes. Decorated definitions are unwrapped so the
//! function/class underneath is seen (the decorator lines are read by the
//! deprecation scan).

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children};
use crate::surface::rules::common::{qual, symbol, vis, WHOLE};
use tree_sitter::Node;

pub fn symbols(root: Node, src: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for child in named_children(root) {
        item(child, "", src, &mut out);
    }
    out
}

fn item(node: Node, prefix: &str, src: &str, out: &mut Vec<Symbol>) {
    let n = unwrap(node);
    match n.kind() {
        "function_definition" => {
            let name = field_text(n, src, "name").unwrap_or_default();
            let kind = if prefix.is_empty() { "Function" } else { "Method" };
            out.push(symbol(kind, qual(prefix, name), n, "body", vis(public(name)), src));
        }
        "class_definition" => {
            let name = field_text(n, src, "name").unwrap_or_default();
            out.push(symbol("Class", qual(prefix, name), n, "body", vis(public(name)), src));
            let cq = qual(prefix, name);
            for m in body_members(n) {
                member(m, &cq, public(name), src, out);
            }
        }
        _ => {}
    }
}

fn member(node: Node, cq: &str, cls_public: bool, src: &str, out: &mut Vec<Symbol>) {
    let n = unwrap(node);
    if n.kind() == "function_definition" {
        let name = field_text(n, src, "name").unwrap_or_default();
        out.push(symbol("Method", qual(cq, name), n, "body", vis(cls_public && public(name)), src));
    } else if n.kind() == "expression_statement" {
        attribute(n, cq, cls_public, src, out);
    }
}

/// A typed class attribute (`name: Type [= ...]`) — part of the data contract.
fn attribute(stmt: Node, cq: &str, cls_public: bool, src: &str, out: &mut Vec<Symbol>) {
    for a in named_children(stmt) {
        if a.kind() == "assignment" && a.child_by_field_name("type").is_some() {
            if let Some(name) = field_text(a, src, "left") {
                let body = if a.child_by_field_name("right").is_some() { "right" } else { WHOLE };
                out.push(symbol("Field", qual(cq, name), a, body, vis(cls_public && public(name)), src));
            }
        }
    }
}

fn unwrap(n: Node) -> Node {
    if n.kind() == "decorated_definition" {
        n.child_by_field_name("definition").unwrap_or(n)
    } else {
        n
    }
}

fn public(name: &str) -> bool {
    !name.starts_with('_')
}

fn body_members(n: Node) -> Vec<Node> {
    n.child_by_field_name("body").map(named_children).unwrap_or_default()
}
