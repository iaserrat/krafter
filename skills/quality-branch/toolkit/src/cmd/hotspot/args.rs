use crate::util::defaults::{ALL_HISTORY, DEFAULT_TOP};
use clap::Args as ClapArgs;

/// `cqt hotspot` — rank files by churn x complexity; flag the ones this branch
/// touches (you are editing already-fragile code).
#[derive(ClapArgs)]
pub struct Args {
    /// Commits to consider for churn (0 = all history).
    #[arg(long, default_value_t = ALL_HISTORY)]
    pub window: usize,

    /// Number of top-ranked files to return.
    #[arg(long, default_value_t = DEFAULT_TOP)]
    pub top: usize,
}
