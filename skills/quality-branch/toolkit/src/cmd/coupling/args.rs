use crate::util::defaults::DEFAULT_TOP;
use clap::Args as ClapArgs;

const MIN_SHARED: usize = 5;
const MIN_REVS: usize = 5;
const MIN_DEGREE: f64 = 0.30;
const MAX_CHANGESET: usize = 30;

/// `cqt coupling` — files that historically change together. Default scope
/// "branch" surfaces the missing co-change: "you changed A but not B, which
/// usually rides along". "global" is a whole-history report.
#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = "branch")]
    pub scope: String,
    /// Denominator for degree: "min" (hidden-dependency) or "avg" (Code Maat).
    #[arg(long, default_value = "min")]
    pub denominator: String,
    #[arg(long, default_value_t = MIN_SHARED)]
    pub min_shared: usize,
    #[arg(long, default_value_t = MIN_REVS)]
    pub min_revs: usize,
    #[arg(long, default_value_t = MIN_DEGREE)]
    pub min_degree: f64,
    #[arg(long, default_value_t = MAX_CHANGESET)]
    pub max_changeset: usize,
    #[arg(long, default_value_t = DEFAULT_TOP)]
    pub top: usize,
}
