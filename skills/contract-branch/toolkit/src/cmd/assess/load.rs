//! Build the breaking-change candidate list: for every changed file, extract
//! the base-side contract (from the git blob) and the head-side contract (from
//! the working tree), then diff them. Files the engine parsed but ctk has no
//! contract rule for are reported unmeasured, never as a silent empty contract.

use crate::cmd::assess::classify;
use crate::cmd::assess::model::Candidate;
use crate::cmd::{self, Ctx};
use crate::git::{self, diff::ChangedFile};
use serde::Serialize;

const NO_RULE: &str = "language parsed but ctk has no contract rule (e.g. Go, C/C++)";

/// A changed file whose contract ctk could not read (so findings may be missed).
#[derive(Serialize)]
pub struct Unmeasured {
    pub path: String,
    pub reason: &'static str,
}

/// Everything the report needs: ordered candidates and the honesty list.
pub struct Loaded {
    pub candidates: Vec<Candidate>,
    pub unmeasured: Vec<Unmeasured>,
}

pub fn load(ctx: &Ctx) -> anyhow::Result<Loaded> {
    let changed = git::changed_files(&ctx.repo, &ctx.base)?;
    let mut candidates = Vec::new();
    let mut unmeasured = Vec::new();
    for cf in &changed {
        let path = anchor_path(cf);
        let before = cmd::surface_at(&ctx.repo, &ctx.base, &cf.old);
        let after = cmd::surface_worktree(&ctx.repo, &cf.new);
        if after.parse_ok && !after.vis_supported {
            unmeasured.push(Unmeasured {
                path: path.clone(),
                reason: NO_RULE,
            });
        }
        candidates.extend(classify::compare(&before.symbols, &after.symbols, &path));
    }
    candidates.sort_by(|x, y| {
        y.breaking
            .cmp(&x.breaking)
            .then(x.reason.rank().cmp(&y.reason.rank()))
            .then_with(|| x.path.cmp(&y.path))
            .then(x.line.cmp(&y.line))
            .then_with(|| x.symbol.cmp(&y.symbol))
    });
    unmeasured.sort_by(|x, y| x.path.cmp(&y.path));
    Ok(Loaded {
        candidates,
        unmeasured,
    })
}

/// Removals carry an empty head path; anchor them at the base-side path.
fn anchor_path(cf: &ChangedFile) -> String {
    if cf.new.is_empty() {
        cf.old.clone()
    } else {
        cf.new.clone()
    }
}
