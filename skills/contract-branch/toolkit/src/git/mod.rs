//! Thin, deterministic git wrappers via std::process (no git crate). Every
//! signal here is a function of repo state, so the same repo yields the same
//! numbers — that is the determinism guarantee the skill leans on.

pub mod blob;
pub mod diff;
pub mod repo;

pub use blob::blob_at;
pub use diff::changed_files;
pub use repo::{default_base, toplevel};

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Run `git -C <repo> <args>` and return stdout, or bail with stderr.
/// `core.quotepath=false` keeps UTF-8 paths literal so they compare
/// byte-identically across git invocations (no octal-escaping skew).
pub fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "core.quotepath=false"])
        .args(args)
        .output()
        .context("failed to invoke git (is it installed?)")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
