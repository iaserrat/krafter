//! Change frequency per file: the count of commits touching each path. Per
//! Tornhill/CodeScene this is the single most validated quality signal, and it
//! is fully deterministic from history (no wall-clock, no dates).

use crate::util::defaults::ALL_HISTORY;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Commits-touching-file counts. `window=0` means all history; otherwise only
/// the most recent `window` commits are considered (reproducible given state).
pub fn commit_counts(repo: &Path, window: usize) -> Result<HashMap<String, usize>> {
    let limit = format!("-n{window}");
    let mut args = vec!["log", "--format=", "--name-only"];
    if window != ALL_HISTORY {
        args.push(&limit);
    }
    let out = super::git(repo, &args)?;
    let mut counts = HashMap::new();
    for line in out.lines() {
        if line.is_empty() {
            continue;
        }
        *counts.entry(line.to_string()).or_insert(0) += 1;
    }
    Ok(counts)
}
