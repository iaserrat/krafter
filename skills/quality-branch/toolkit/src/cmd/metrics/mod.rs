//! Per-file, per-function metrics. The base primitive: delta/calibrate/assess
//! are this measurement applied across git refs and the whole repo.

pub mod args;
pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::engine::{self, scan, FileAnalysis};
use serde_json::json;
use std::path::Path;

const SKIPPED: &str = "skipped";

pub fn run(a: Args, _ctx: &Ctx) -> anyhow::Result<()> {
    let mut files = Vec::new();
    for p in &a.paths {
        match scan::read_source(Path::new(p)) {
            Some(bytes) => files.push(engine::analyze(Path::new(p), bytes)),
            None => files.push(skipped(p)),
        }
    }
    let parse_errors = files.iter().filter(|f| !f.parse_ok).count();
    cmd::emit(&json!({
        "command": "metrics",
        "files": files,
        "parse_errors": parse_errors,
    }));
    Ok(())
}

/// A file we could not read/parse (oversized, binary, or missing).
fn skipped(path: &str) -> FileAnalysis {
    FileAnalysis {
        path: path.to_string(),
        lang: SKIPPED.to_string(),
        parse_ok: false,
        functions: Vec::new(),
    }
}
