//! Per-file public-contract surface. The base primitive: `assess` is this
//! extraction applied across two git refs and diffed.

pub mod args;
pub use args::Args;

use crate::cmd::{self, Ctx};
use crate::util::source;
use crate::surface::{self, FileSurface};
use serde_json::json;
use std::path::Path;

const SKIPPED: &str = "skipped";

pub fn run(a: Args, _ctx: &Ctx) -> anyhow::Result<()> {
    let mut files = Vec::new();
    for p in &a.paths {
        match source::read_source(Path::new(p)) {
            Some(bytes) => files.push(surface::extract(Path::new(p), bytes)),
            None => files.push(skipped(p)),
        }
    }
    let unmeasured = files
        .iter()
        .filter(|f| !f.parse_ok || !f.vis_supported)
        .count();
    cmd::emit(&json!({
        "command": "surface",
        "schema": "ctk.surface/v1",
        "files": files,
        "unmeasured": unmeasured,
    }));
    Ok(())
}

/// A file we could not read (oversized, binary, or missing).
fn skipped(path: &str) -> FileSurface {
    FileSurface {
        path: path.to_string(),
        lang: SKIPPED.to_string(),
        parse_ok: false,
        vis_supported: false,
        symbols: Vec::new(),
    }
}
