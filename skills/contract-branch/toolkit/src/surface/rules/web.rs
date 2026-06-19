//! TS/JS contract rules. A top-level declaration is public when wrapped in
//! `export`; class/interface/enum members are in `web_member`. JS/JSX are parsed
//! by the TSX grammar (a superset), so the same rules apply.

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children};
use crate::surface::rules::common::{symbol, vis, WHOLE};
use crate::surface::rules::web_member as member;
use tree_sitter::Node;

pub fn symbols(root: Node, src: &str) -> Vec<Symbol> {
    let mut out = Vec::new();
    for child in named_children(root) {
        if child.kind() == "export_statement" {
            if let Some(decl) = child.child_by_field_name("declaration") {
                item(decl, true, src, &mut out);
            }
        } else {
            item(child, false, src, &mut out);
        }
    }
    out
}

fn item(decl: Node, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    match decl.kind() {
        "class_declaration" | "abstract_class_declaration" => member::class(decl, exported, src, out),
        "interface_declaration" => member::interface(decl, exported, src, out),
        "enum_declaration" => member::enumeration(decl, exported, src, out),
        "function_declaration" | "generator_function_declaration" => {
            out.push(named(decl, "Function", "body", exported, src))
        }
        "type_alias_declaration" => out.push(named(decl, "TypeAlias", WHOLE, exported, src)),
        "lexical_declaration" | "variable_declaration" => constants(decl, exported, src, out),
        _ => {}
    }
}

fn named(n: Node, kind: &str, body: &str, exported: bool, src: &str) -> Symbol {
    let name = field_text(n, src, "name").unwrap_or_default();
    symbol(kind, name.to_string(), n, body, vis(exported), src)
}

fn constants(n: Node, exported: bool, src: &str, out: &mut Vec<Symbol>) {
    for d in named_children(n) {
        if d.kind() == "variable_declarator" {
            if let Some(name) = field_text(d, src, "name") {
                out.push(symbol("Const", name.to_string(), d, "value", vis(exported), src));
            }
        }
    }
}
