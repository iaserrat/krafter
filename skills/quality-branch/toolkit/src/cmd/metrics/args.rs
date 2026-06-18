use clap::Args as ClapArgs;

/// `cqt metrics --paths a.rs b.py` — per-function metrics for given files.
#[derive(ClapArgs)]
pub struct Args {
    /// Files to measure (working-tree contents).
    #[arg(long, required = true, num_args = 1..)]
    pub paths: Vec<String>,
}
