//! ctk — contract toolkit. Deterministic measurement driven by the
//! `contract-branch` skill: it extracts a public API surface and diffs it
//! across refs into reason-coded breaking-change candidates; the skill judges.
//!
//! Design contract (so an agent can drive it):
//!   - stdout = a single JSON document with the result (parse this)
//!   - stderr = human-readable progress / warnings
//!   - deterministic: every signal is a function of repo state, no wall-clock
mod cmd;
mod git;
mod surface;
mod util;

use clap::{Parser, Subcommand};
use cmd::Ctx;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ctk",
    version,
    about = "Contract toolkit — deterministic breaking-change detection for the contract-branch skill"
)]
struct Cli {
    /// Repo root (default: git toplevel of the current directory).
    #[arg(long, global = true)]
    repo: Option<PathBuf>,

    /// Base ref to measure the branch against (default: origin/HEAD or main).
    #[arg(long, global = true)]
    base: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// The spine: ranked, reason-coded breaking-change candidates vs base.
    Assess(cmd::assess::Args),
    /// The public contract surface of given files (no git).
    Surface(cmd::surface::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repo = git::toplevel(cli.repo)?;
    let base = cli.base.unwrap_or_else(|| git::default_base(&repo));
    let ctx = Ctx { repo, base };
    match cli.cmd {
        Cmd::Assess(a) => cmd::assess::run(a, &ctx),
        Cmd::Surface(a) => cmd::surface::run(a, &ctx),
    }
}
