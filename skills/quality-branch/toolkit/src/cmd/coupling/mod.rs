//! Temporal coupling from git history, in the "missing co-change" framing: a
//! pair where a branch-changed file is coupled to one the branch did NOT touch
//! is a likely-forgotten edit. Deterministic — counts of co-occurrence, no dates.
//! In branch scope the tally runs over history UP TO the base ref, so the
//! branch's own commits don't bias its own advice.

pub mod args;
pub mod compute;
pub mod log;
pub mod model;

pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::git;
use crate::util::defaults::HEAD_REF;
use anyhow::Result;
use model::Couple;
use serde_json::json;
use std::cmp::Ordering;
use std::collections::HashSet;

const SCOPE_BRANCH: &str = "branch";
const MISSED: &str = "missed";
const COVERED: &str = "covered";

pub fn run(a: Args, ctx: &Ctx) -> Result<()> {
    let branch_scope = a.scope == SCOPE_BRANCH;
    let rev = if branch_scope { ctx.base.as_str() } else { HEAD_REF };
    let commits = log::commit_filesets(&ctx.repo, rev, a.max_changeset)?;
    let tally = compute::tally(&commits);
    let mut couples = compute::couples(&tally, a.min_shared, a.min_revs, a.min_degree, a.denominator != "avg");

    if branch_scope {
        let branch = branch_paths(ctx);
        couples.retain(|c| branch.contains(&c.file_a) || branch.contains(&c.file_b));
        couples.iter_mut().for_each(|c| annotate(c, &branch));
    }
    couples.sort_by(|x, y| order(x, y, branch_scope));
    couples.truncate(a.top);

    cmd::emit(&json!({
        "command": "coupling",
        "schema": "cqt.coupling/v1",
        "scope": a.scope,
        "denominator": a.denominator,
        "commits_analyzed": commits.len(),
        "pairs": couples,
    }));
    Ok(())
}

/// Both sides of every change, so a renamed file is recognized under the
/// historical (old) path its couplings are keyed on, not only its new path.
fn branch_paths(ctx: &Ctx) -> HashSet<String> {
    let mut set = HashSet::new();
    for cf in git::changed_files(&ctx.repo, &ctx.base).unwrap_or_default() {
        set.insert(cf.old);
        set.insert(cf.new);
    }
    set
}

/// Mark the branch-side file as the anchor; "missed" if its partner was not
/// touched by the branch (the actionable signal), else "covered".
fn annotate(c: &mut Couple, branch: &HashSet<String>) {
    let a_in = branch.contains(&c.file_a);
    let b_in = branch.contains(&c.file_b);
    if a_in && b_in {
        c.status = COVERED.into();
        c.anchor = c.file_a.clone();
    } else if a_in {
        c.status = MISSED.into();
        c.anchor = c.file_a.clone();
    } else {
        c.status = MISSED.into();
        c.anchor = c.file_b.clone();
    }
}

/// Total order: missed first (branch scope), then degree desc, then file names.
fn order(x: &Couple, y: &Couple, branch_scope: bool) -> Ordering {
    if branch_scope {
        let xm = (x.status != MISSED) as u8;
        let ym = (y.status != MISSED) as u8;
        if xm != ym {
            return xm.cmp(&ym);
        }
    }
    y.degree
        .partial_cmp(&x.degree)
        .unwrap_or(Ordering::Equal)
        .then_with(|| x.file_a.cmp(&y.file_a))
        .then_with(|| x.file_b.cmp(&y.file_b))
}
