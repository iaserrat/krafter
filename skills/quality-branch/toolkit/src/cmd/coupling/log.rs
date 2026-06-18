//! Extract per-commit file sets from git history. NUL-delimited records make
//! parsing unambiguous for paths with spaces; merges and mega-commits are
//! dropped (they fabricate dense coupling). Deterministic: no dates enter.

use crate::git;
use anyhow::Result;
use std::path::Path;

const RENAMES: &str = "--find-renames=40%";
const FORMAT: &str = "--format=%x00%H";
/// Number of leading lines in a record that are the commit SHA, not a path.
const SHA_LINE: usize = 1;

/// Sorted, deduped file set per commit, excluding merges and any commit that
/// touches more than `max_changeset` files.
pub fn commit_filesets(repo: &Path, rev: &str, max_changeset: usize) -> Result<Vec<Vec<String>>> {
    let out = git::git(repo, &["log", "--no-merges", "--name-only", RENAMES, FORMAT, rev])?;
    let mut commits = Vec::new();
    for chunk in out.split('\u{0}') {
        let mut files: Vec<String> = chunk
            .lines()
            .skip(SHA_LINE)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        files.sort();
        files.dedup();
        if files.is_empty() || files.len() > max_changeset {
            continue;
        }
        commits.push(files);
    }
    Ok(commits)
}
