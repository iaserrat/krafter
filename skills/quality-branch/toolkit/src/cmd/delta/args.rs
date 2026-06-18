use clap::Args as ClapArgs;

/// `cqt delta` — before/after metrics for functions the branch changed,
/// measured `base...HEAD`. Uses the global --base/--repo; no local flags.
#[derive(ClapArgs)]
pub struct Args {}
