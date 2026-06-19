//! Resolve the repo root and the base ref to diff against.

use crate::util::defaults::{MAIN, MASTER};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repo root: an explicit `--repo`, else `git rev-parse`, else the cwd.
pub fn toplevel(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(o) if o.status.success() => {
            Ok(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        }
        _ => std::env::current_dir().context("not a git repo; pass --repo"),
    }
}

/// Detected default branch: `origin/HEAD`, else `main`, else `master`.
pub fn default_base(repo: &Path) -> String {
    if let Some(b) = origin_head(repo) {
        return b;
    }
    for cand in [MAIN, MASTER] {
        if rev_exists(repo, cand) {
            return cand.to_string();
        }
    }
    MAIN.to_string()
}

fn origin_head(repo: &Path) -> Option<String> {
    let out = super::git(repo, &["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"]).ok()?;
    out.trim().strip_prefix("refs/remotes/origin/").map(String::from)
}

fn rev_exists(repo: &Path, rev: &str) -> bool {
    super::git(repo, &["rev-parse", "--verify", "--quiet", rev]).is_ok()
}
