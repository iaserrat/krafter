//! Go type members: struct fields and interface methods. A member is public
//! only if both the member name and its type are exported (capitalized).

use crate::surface::model::Symbol;
use crate::surface::node::{field_text, named_children};
use crate::surface::rules::common::{capitalized, qual, symbol, vis, WHOLE};
use tree_sitter::Node;

pub fn types(n: Node, src: &str, out: &mut Vec<Symbol>) {
    for spec in named_children(n) {
        match spec.kind() {
            "type_spec" => type_spec(spec, src, out),
            "type_alias" => {
                if let Some(name) = field_text(spec, src, "name") {
                    out.push(symbol("TypeAlias", name.to_string(), spec, WHOLE, vis(capitalized(name)), src));
                }
            }
            _ => {}
        }
    }
}

fn type_spec(spec: Node, src: &str, out: &mut Vec<Symbol>) {
    let name = field_text(spec, src, "name").unwrap_or_default();
    let public = capitalized(name);
    let ty = spec.child_by_field_name("type");
    let kind = match ty.map(|t| t.kind()) {
        Some("struct_type") => "Struct",
        Some("interface_type") => "Interface",
        _ => "Type",
    };
    out.push(symbol(kind, name.to_string(), spec, "type", vis(public), src));
    match ty.map(|t| t.kind()) {
        Some("struct_type") => fields(ty.unwrap(), name, public, src, out),
        Some("interface_type") => methods(ty.unwrap(), name, public, src, out),
        _ => {}
    }
}

fn fields(struct_ty: Node, ty: &str, public: bool, src: &str, out: &mut Vec<Symbol>) {
    let Some(list) = named_children(struct_ty).into_iter().next() else {
        return;
    };
    for f in named_children(list) {
        if f.kind() == "field_declaration" {
            if let Some(fname) = field_text(f, src, "name") {
                out.push(symbol("Field", qual(ty, fname), f, WHOLE, vis(public && capitalized(fname)), src));
            }
        }
    }
}

fn methods(iface: Node, ty: &str, public: bool, src: &str, out: &mut Vec<Symbol>) {
    for m in named_children(iface) {
        if m.kind() == "method_elem" {
            if let Some(mname) = field_text(m, src, "name") {
                out.push(symbol("Method", qual(ty, mname), m, WHOLE, vis(public && capitalized(mname)), src));
            }
        }
    }
}
