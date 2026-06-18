//! Behavioral signal: files that are both complex and frequently changed are
//! where defects concentrate. Branch-touched hotspots are the ones to review
//! hardest.

pub mod args;
pub mod model;
pub mod score;

pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::git;
use anyhow::Result;
use serde_json::json;
use std::collections::HashSet;

pub fn run(a: Args, ctx: &Ctx) -> Result<()> {
    let churn = git::commit_counts(&ctx.repo, a.window)?;
    let files = cmd::analyze_tracked(&ctx.repo)?;
    let branch: HashSet<String> = git::changed_paths(&ctx.repo, &ctx.base)
        .unwrap_or_default()
        .into_iter()
        .collect();
    let hotspots = score::rank(&files, &churn, &branch, a.top);
    cmd::emit(&json!({
        "command": "hotspot",
        "window": a.window,
        "files_analyzed": files.len(),
        "hotspots": hotspots,
    }));
    Ok(())
}
