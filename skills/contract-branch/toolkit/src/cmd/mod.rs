pub mod assess;
pub mod surface;

use crate::util::source;
use crate::git;
use crate::surface::{extract, FileSurface};
use std::path::{Path, PathBuf};

/// Run context shared by every command: the repo root and the base ref the
/// branch is measured against.
pub struct Ctx {
    pub repo: PathBuf,
    pub base: String,
}

const ABSENT: &str = "absent";

/// Contract surface of `path` at git `rev` (the base side of a comparison).
/// A missing/binary/oversized blob yields an empty surface, so a newly added
/// file simply has nothing on its base side.
pub fn surface_at(repo: &Path, rev: &str, path: &str) -> FileSurface {
    git::blob_at(repo, rev, path)
        .and_then(source::guard_bytes)
        .map(|b| extract(Path::new(path), b))
        .unwrap_or_else(|| absent(path))
}

/// Contract surface of `path` in the working tree (the after side). Empty if the
/// file is gone (deleted) or unreadable.
pub fn surface_worktree(repo: &Path, path: &str) -> FileSurface {
    source::read_source(&repo.join(path))
        .map(|b| extract(Path::new(path), b))
        .unwrap_or_else(|| absent(path))
}

fn absent(path: &str) -> FileSurface {
    FileSurface {
        path: path.to_string(),
        lang: ABSENT.to_string(),
        parse_ok: false,
        vis_supported: false,
        symbols: Vec::new(),
    }
}

/// Emit the single JSON result document to stdout (the agent parses this).
pub fn emit(v: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
    );
}
