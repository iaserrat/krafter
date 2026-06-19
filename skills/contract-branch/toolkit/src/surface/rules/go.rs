//! Go contract rules. Public = an exported (capitalized) identifier. Captures
//! functions, methods (qualified by receiver type), constants and package vars;
//! struct/interface types and their members are in `go_member`.

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children, text};
use crate::surface::rules::common::{capitalized, qual, symbol, vis};
use crate::surface::rules::go_member as member;
use tree_sitter::Node;

pub fn symbols(root: Node, src: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for child in named_children(root) {
        match child.kind() {
            "function_declaration" => out.push(func(child, src)),
            "method_declaration" => out.push(method(child, src)),
            "type_declaration" => member::types(child, src, &mut out),
            "const_declaration" | "var_declaration" => specs(child, src, &mut out),
            _ => {}
        }
    }
    out
}

fn func(n: Node, src: &str) -> Symbol {
    let name = field_text(n, src, "name").unwrap_or_default();
    symbol("Function", name.to_string(), n, "body", vis(capitalized(name)), src)
}

fn method(n: Node, src: &str) -> Symbol {
    let name = field_text(n, src, "name").unwrap_or_default();
    let recv = receiver_type(n, src);
    symbol("Method", qual(&recv, name), n, "body", vis(capitalized(name)), src)
}

/// The receiver's base type (`(c *Config)` → `Config`), for the qualified name.
fn receiver_type(n: Node, src: &str) -> String {
    n.child_by_field_name("receiver")
        .and_then(|r| named_children(r).into_iter().next())
        .and_then(|p| p.child_by_field_name("type"))
        .map(|t| text(t, src).trim_start_matches('*').trim().to_string())
        .unwrap_or_default()
}

fn specs(n: Node, src: &str, out: &mut Vec<Symbol>) {
    for spec in named_children(n) {
        if let Some(name) = field_text(spec, src, "name") {
            out.push(symbol("Const", name.to_string(), spec, "value", vis(capitalized(name)), src));
        }
    }
}
