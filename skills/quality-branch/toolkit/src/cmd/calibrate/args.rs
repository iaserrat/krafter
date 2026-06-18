use clap::Args as ClapArgs;

/// `cqt calibrate` — the repo's own metric distribution (percentiles), so
/// findings read as "p99 complexity for THIS repo", never an absolute threshold.
#[derive(ClapArgs)]
pub struct Args {}
