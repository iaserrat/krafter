//! The headline complexity command: what did this branch do to per-function
//! complexity? Compares committed `base...HEAD` (rename-aware) so the result is
//! reproducible. Sorted by cognitive increase, worst first.

pub mod args;
pub mod model;
mod pair;

pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::engine;
use crate::git;
use crate::util::defaults::HEAD_REF;
use anyhow::Result;
use serde_json::json;
use std::cmp::Ordering;

pub fn run(_a: Args, ctx: &Ctx) -> Result<()> {
    let changed = git::changed_files(&ctx.repo, &ctx.base)?;
    let mut deltas = Vec::new();
    let mut analyzed = 0usize;
    for cf in &changed {
        let before = cmd::functions_at(&ctx.repo, &ctx.base, &cf.old);
        let after = cmd::functions_at(&ctx.repo, HEAD_REF, &cf.new);
        if !after.is_empty() {
            analyzed += 1;
        }
        for p in engine::pair::pair_functions(&cf.new, &before, &after) {
            if let Some(d) = pair::to_delta(&p) {
                deltas.push(d);
            }
        }
    }
    deltas.sort_by(|x, y| {
        y.delta_cognitive
            .partial_cmp(&x.delta_cognitive)
            .unwrap_or(Ordering::Equal)
            .then_with(|| x.path.cmp(&y.path))
            .then_with(|| x.name.cmp(&y.name))
    });
    cmd::emit(&json!({
        "command": "delta",
        "base": ctx.base,
        "changed_files": changed.len(),
        "files_analyzed": analyzed,
        "functions": deltas,
    }));
    Ok(())
}
