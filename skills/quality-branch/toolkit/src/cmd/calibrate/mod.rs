//! Repo-relative calibration. Feeds the skill's ranking so a delta is judged
//! against this codebase's own complexity distribution, not folklore thresholds.

pub mod args;
pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::util::stats;
use anyhow::Result;
use serde_json::json;

pub fn run(_a: Args, ctx: &Ctx) -> Result<()> {
    let files = cmd::analyze_tracked(&ctx.repo)?;
    let mut cognitive = Vec::new();
    let mut cyclomatic = Vec::new();
    for f in &files {
        for m in &f.functions {
            cognitive.push(m.cognitive);
            cyclomatic.push(m.cyclomatic);
        }
    }
    cmd::emit(&json!({
        "command": "calibrate",
        "files_analyzed": files.len(),
        "functions": cognitive.len(),
        "cognitive": stats::distribution(cognitive),
        "cyclomatic": stats::distribution(cyclomatic),
    }));
    Ok(())
}
