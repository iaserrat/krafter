//! Thin helpers over tree-sitter `Node`: text slicing, field/child access, and
//! deriving a declaration's signature (its source up to the body). Shared by
//! every per-language rule set so the walks stay small.

use tree_sitter::Node;

/// The source text spanned by `n`.
pub fn text<'a>(n: Node, src: &'a str) -> &'a str {
    &src[n.start_byte()..n.end_byte()]
}

/// Text of the named field `field`, if present.
pub fn field_text<'a>(n: Node, src: &'a str, field: &str) -> Option<&'a str> {
    n.child_by_field_name(field).map(|c| text(c, src))
}

/// Named children of `n`.
pub fn named_children<'a>(n: Node<'a>) -> Vec<Node<'a>> {
    let mut cur = n.walk();
    n.named_children(&mut cur).collect()
}

/// A declaration's signature: its source up to the `body` field (the block),
/// or the whole node if it has no such field. Stops before a member body so a
/// type/function header is captured without its implementation.
pub fn header(n: Node, src: &str, body_field: &str) -> String {
    let end = n
        .child_by_field_name(body_field)
        .map(|b| b.start_byte())
        .unwrap_or_else(|| n.end_byte());
    src[n.start_byte()..end].to_string()
}
