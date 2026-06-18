//! Read a file's bytes at a specific ref, without touching the working tree.

use std::path::Path;
use std::process::Command;

/// Bytes of `<rev>:<path>`, or None if the file did not exist at that ref
/// (e.g. a file the branch newly added has no base-side blob).
pub fn blob_at(repo: &Path, rev: &str, path: &str) -> Option<Vec<u8>> {
    let spec = format!("{rev}:{path}");
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", &spec])
        .output()
        .ok()?;
    if out.status.success() {
        Some(out.stdout)
    } else {
        None
    }
}
