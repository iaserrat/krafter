//! Diff a file's base-side and head-side contract surfaces into reason-coded
//! candidates. Pure: same two surfaces always yield the same ordered list.

use crate::cmd::assess::model::{Candidate, Reason};
use crate::surface::{sig, Symbol};
use std::collections::BTreeMap;

/// Ordered candidates for one file (breaking first, then by line, then name).
pub fn compare(before: &[Symbol], after: &[Symbol], path: &str) -> Vec<Candidate> {
    let after_by = index(after);
    let before_by = index(before);
    let mut out = Vec::new();
    for b in before.iter().filter(|s| s.is_public()) {
        if let Some(c) = regression(b, after_by.get(b.name.as_str()).copied(), path) {
            out.push(c);
        }
    }
    for a in after.iter().filter(|s| s.is_public()) {
        let was_public = before_by.get(a.name.as_str()).is_some_and(|s| s.is_public());
        if !was_public {
            out.push(mk(Reason::Added, a, path, None, Some(a.signature.clone())));
        }
    }
    out.sort_by(|x, y| {
        y.breaking
            .cmp(&x.breaking)
            .then(x.line.cmp(&y.line))
            .then_with(|| x.symbol.cmp(&y.symbol))
    });
    out
}

fn index(symbols: &[Symbol]) -> BTreeMap<&str, &Symbol> {
    symbols.iter().map(|s| (s.name.as_str(), s)).collect()
}

/// A public base symbol vs its head counterpart: removed, restricted, changed,
/// or (None) unchanged.
fn regression(b: &Symbol, after: Option<&Symbol>, path: &str) -> Option<Candidate> {
    match after {
        None if b.deprecated => Some(at_base(Reason::RemovedDeprecated, b, path)),
        None => Some(at_base(Reason::Removed, b, path)),
        Some(a) if !a.is_public() => {
            Some(mk(Reason::VisibilityReduced, a, path, before(b), after_sig(a)))
        }
        Some(a) if sig::canonical(&a.signature) != sig::canonical(&b.signature) => {
            Some(mk(Reason::SignatureChanged, a, path, before(b), after_sig(a)))
        }
        Some(_) => None,
    }
}

fn mk(
    reason: Reason,
    anchor: &Symbol,
    path: &str,
    before: Option<String>,
    after: Option<String>,
) -> Candidate {
    Candidate {
        path: path.to_string(),
        symbol: anchor.name.clone(),
        kind: anchor.kind.clone(),
        line: anchor.start_line,
        reason,
        semver: reason.semver(),
        breaking: reason.breaking(),
        suppressed: false,
        before,
        after,
    }
}

/// A removal anchors at the base-side symbol (it has no head-side line).
fn at_base(reason: Reason, b: &Symbol, path: &str) -> Candidate {
    mk(reason, b, path, before(b), None)
}

fn before(b: &Symbol) -> Option<String> {
    Some(b.signature.clone())
}

fn after_sig(a: &Symbol) -> Option<String> {
    Some(a.signature.clone())
}
