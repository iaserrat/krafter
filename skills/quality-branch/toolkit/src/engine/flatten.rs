//! Flatten the rust-code-analysis FuncSpace tree into a flat list of named
//! functions. Container spaces (Unit/Class/Impl) only contribute their name to
//! the qualified path; we emit a row per Function. Output is sorted by
//! (start_line, name) so it is deterministic, not parser-incidental.

use crate::engine::model::FunctionMetrics;
use crate::util::defaults::ANON_FN;
use rust_code_analysis::{FuncSpace, SpaceKind};

pub fn flatten(root: &FuncSpace) -> Vec<FunctionMetrics> {
    let mut out = Vec::new();
    walk(root, "", &mut out);
    out.sort_by(|a, b| {
        a.start_line
            .cmp(&b.start_line)
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

fn walk(space: &FuncSpace, prefix: &str, out: &mut Vec<FunctionMetrics>) {
    let name = space.name.clone().unwrap_or_else(|| ANON_FN.to_string());
    let qual = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}::{name}")
    };
    if matches!(space.kind, SpaceKind::Function) {
        let m = &space.metrics;
        out.push(FunctionMetrics {
            name: qual.clone(),
            kind: format!("{:?}", space.kind),
            start_line: space.start_line,
            end_line: space.end_line,
            cyclomatic: m.cyclomatic.cyclomatic(),
            cognitive: m.cognitive.cognitive(),
            sloc: m.loc.sloc(),
            params: m.nargs.fn_args(),
            exits: m.nexits.exit(),
        });
    }
    // The file unit and unknown wrappers contribute no name segment.
    let child_prefix = match space.kind {
        SpaceKind::Unit | SpaceKind::Unknown => prefix.to_string(),
        _ => qual,
    };
    for child in &space.spaces {
        walk(child, &child_prefix, out);
    }
}
