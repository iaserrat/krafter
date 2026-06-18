//! cqt — code-quality toolkit. Deterministic measurement driven by the
//! `quality-branch` skill: it computes numbers, the skill does the judging.
//!
//! Design contract (so an agent can drive it):
//!   - stdout = a single JSON document with the result (parse this)
//!   - stderr = human-readable progress / warnings
//!   - deterministic: every signal is a function of repo state, no wall-clock
mod cmd;
mod engine;
mod git;
mod util;

use clap::{Parser, Subcommand};
use cmd::Ctx;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cqt",
    version,
    about = "Code-quality toolkit — deterministic metrics for the quality-branch skill"
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
    /// Unified branch quality: ranked, reason-coded candidate findings.
    Assess(cmd::assess::Args),
    /// Per-function metrics for given files.
    Metrics(cmd::metrics::Args),
    /// Before/after complexity for functions the branch changed.
    Delta(cmd::delta::Args),
    /// Rank files by churn x complexity; flag branch-touched hotspots.
    Hotspot(cmd::hotspot::Args),
    /// The repo's own metric distribution (percentiles) for ranking.
    Calibrate(cmd::calibrate::Args),
    /// Detect duplication the branch introduces (cross-file clones).
    Dup(cmd::dup::Args),
    /// Temporal coupling: files that historically co-change but the branch missed.
    Coupling(cmd::coupling::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repo = git::toplevel(cli.repo)?;
    let base = cli.base.unwrap_or_else(|| git::default_base(&repo));
    let ctx = Ctx { repo, base };
    match cli.cmd {
        Cmd::Assess(a) => cmd::assess::run(a, &ctx),
        Cmd::Metrics(a) => cmd::metrics::run(a, &ctx),
        Cmd::Delta(a) => cmd::delta::run(a, &ctx),
        Cmd::Hotspot(a) => cmd::hotspot::run(a, &ctx),
        Cmd::Calibrate(a) => cmd::calibrate::run(a, &ctx),
        Cmd::Dup(a) => cmd::dup::run(a, &ctx),
        Cmd::Coupling(a) => cmd::coupling::run(a, &ctx),
    }
}
