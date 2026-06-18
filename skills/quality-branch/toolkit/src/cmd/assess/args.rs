use crate::util::defaults::DEFAULT_TOP;
use clap::Args as ClapArgs;

/// `cqt assess` — the unified branch-quality command. Joins delta x
/// repo-percentile x hotspot x biomarkers into ranked, reason-coded candidate
/// findings, each a self-contained evidence bundle citing every raw measurement.
#[derive(ClapArgs)]
pub struct Args {
    /// Max candidate findings to return (ranked, worst first).
    #[arg(long, default_value_t = DEFAULT_TOP)]
    pub top: usize,
}
