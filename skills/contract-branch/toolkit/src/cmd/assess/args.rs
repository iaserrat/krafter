use clap::{Args as ClapArgs, ValueEnum};
use std::path::PathBuf;

/// `ctk assess` — ranked breaking-change candidates for the branch's diff
/// against `--base`. Scope is the global `--repo` / `--base`.
#[derive(ClapArgs)]
pub struct Args {
    /// Exit non-zero when the contract change meets this threshold (CI gate).
    /// Omitted = report-only: always exit 0, the agent reads the JSON.
    #[arg(long, value_enum)]
    pub fail_on: Option<FailOn>,

    /// File of accepted breaking changes to exclude from the gate decision.
    /// One `REASON path symbol` key per line (`#` comments allowed).
    #[arg(long)]
    pub baseline: Option<PathBuf>,
}

/// How strict the CI gate is. `Major` trips only on breaking changes; `Minor`
/// and `Any` trip on any contract change (including additive surface).
#[derive(Clone, Copy, ValueEnum)]
pub enum FailOn {
    Major,
    Minor,
    Any,
}
