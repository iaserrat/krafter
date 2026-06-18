pub mod assess;
pub mod calibrate;
pub mod coupling;
pub mod delta;
pub mod dup;
pub mod hotspot;
pub mod metrics;

use crate::engine::{self, scan, scan::Job, FileAnalysis, FunctionMetrics};
use crate::git;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Run context shared by every command: the repo root and the base ref the
/// branch is measured against.
pub struct Ctx {
    pub repo: PathBuf,
    pub base: String,
}

/// Analyze every tracked file the engine can parse (working-tree contents), in
/// parallel. Unparseable/oversized/binary files are dropped, never counted as
/// zero-complexity. Output is deterministic (sorted by path).
pub fn analyze_tracked(repo: &Path) -> Result<Vec<FileAnalysis>> {
    let listing = git::git(repo, &["ls-files"])?;
    let jobs = listing
        .lines()
        .map(|rel| Job {
            rel: rel.to_string(),
            abs: repo.join(rel),
        })
        .collect();
    Ok(engine::scan::analyze_all(jobs))
}

/// Functions in `path` at git `rev`, guarded for size/binary; empty if the blob
/// is absent or unparseable. Shared by `delta` and `assess`.
pub fn functions_at(repo: &Path, rev: &str, path: &str) -> Vec<FunctionMetrics> {
    git::blob_at(repo, rev, path)
        .and_then(scan::guard_bytes)
        .map(|b| engine::analyze(Path::new(path), b).functions)
        .unwrap_or_default()
}

/// Emit the single JSON result document to stdout (the agent parses this).
pub fn emit(v: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
    );
}
