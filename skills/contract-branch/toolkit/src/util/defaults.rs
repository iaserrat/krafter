//! Named constants — keeps magic literals out of production code (make magic).

/// Git ref for the branch tip we measure against the base.
pub const HEAD_REF: &str = "HEAD";
/// Candidate default branch names, tried in order when none is detected.
pub const MAIN: &str = "main";
pub const MASTER: &str = "master";
