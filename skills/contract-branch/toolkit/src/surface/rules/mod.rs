//! Dispatch to the per-language contract rules. The parse mechanism is uniform
//! (tree-sitter); only these node-kind rules differ per language.

pub mod common;
pub mod go;
pub mod go_member;
pub mod python;
pub mod rust;
pub mod rust_member;
pub mod web;
pub mod web_member;

use crate::surface::lang::Lang;
use crate::surface::model::Symbol;
use tree_sitter::Node;

pub fn symbols(lang: Lang, root: Node, src: &str) -> Vec<Symbol> {
    match lang {
        Lang::Rust => rust::symbols(root, src),
        Lang::Ts | Lang::Tsx => web::symbols(root, src),
        Lang::Python => python::symbols(root, src),
        Lang::Go => go::symbols(root, src),
    }
}
