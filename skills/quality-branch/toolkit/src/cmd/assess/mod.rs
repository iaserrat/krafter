//! The spine: join delta x repo-percentile x hotspot x biomarkers into ranked,
//! reason-coded candidate findings. This is the one command the skill runs to
//! turn four measurement streams into a reviewable, citation-ready artifact.
//!
//! Basis: the repo distribution and hotspot set come from the working tree;
//! before/after deltas come from base/HEAD blobs. Like every branch-diff tool,
//! assess assumes a COMMITTED branch (clean tree), where working tree == HEAD,
//! so all four streams share one basis and the output is a pure function of
//! (repo, base, HEAD).

pub mod args;
pub mod build;
pub mod model;
pub mod reason;
pub mod score;

pub use args::Args;

use crate::cmd::{self, hotspot, Ctx};
use crate::engine::pair;
use crate::git;
use crate::util::defaults::{ALL_HISTORY, HEAD_REF};
use anyhow::Result;
use model::RepoContext;
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashSet;

const HOTSPOT_TOP: usize = 20;

pub fn run(a: Args, ctx: &Ctx) -> Result<()> {
    let scanned = cmd::analyze_tracked(&ctx.repo)?;
    let churn = git::commit_counts(&ctx.repo, ALL_HISTORY)?;
    let mut cognitive_sorted: Vec<f64> = scanned
        .iter()
        .flat_map(|f| f.functions.iter().map(|m| m.cognitive))
        .collect();
    cognitive_sorted.sort_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal));
    let hotspots: HashSet<String> = hotspot::score::rank(&scanned, &churn, &HashSet::new(), HOTSPOT_TOP)
        .into_iter()
        .map(|h| h.path)
        .collect();
    let repo_ctx = RepoContext {
        cognitive_sorted: &cognitive_sorted,
        churn: &churn,
        hotspots: &hotspots,
    };

    let changed = git::changed_files(&ctx.repo, &ctx.base)?;
    let mut cands = Vec::new();
    for cf in &changed {
        let before = cmd::functions_at(&ctx.repo, &ctx.base, &cf.old);
        let after = cmd::functions_at(&ctx.repo, HEAD_REF, &cf.new);
        for p in pair::pair_functions(&cf.new, &before, &after) {
            if let Some(c) = build::build(&p, &repo_ctx) {
                cands.push(c);
            }
        }
    }
    cands.sort_by(|x, y| {
        y.rank
            .score
            .partial_cmp(&x.rank.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| x.path.cmp(&y.path))
            .then_with(|| x.function.cmp(&y.function))
    });
    cands.truncate(a.top);
    cmd::emit(&json!({
        "command": "assess",
        "schema": "cqt.assess/v1",
        "base": ctx.base,
        "changed_files": changed.len(),
        "candidates": cands,
    }));
    Ok(())
}
