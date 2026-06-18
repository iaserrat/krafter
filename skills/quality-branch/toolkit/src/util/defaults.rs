//! Named constants — keeps magic literals out of production code (make magic).

/// Placeholder name for a function space the parser could not name.
pub const ANON_FN: &str = "<anonymous>";
/// Default number of ranked rows a listing command returns.
pub const DEFAULT_TOP: usize = 20;
/// Churn window meaning "all history" (no `-n` cap on git log).
pub const ALL_HISTORY: usize = 0;
/// Git ref for the branch tip we measure against the base.
pub const HEAD_REF: &str = "HEAD";
/// Candidate default branch names, tried in order when none is detected.
pub const MAIN: &str = "main";
pub const MASTER: &str = "master";
/// Percentile scale and the points a Distribution reports.
pub const PCT_MAX: f64 = 100.0;
pub const P50: f64 = 50.0;
pub const P75: f64 = 75.0;
pub const P90: f64 = 90.0;
pub const P95: f64 = 95.0;
pub const P99: f64 = 99.0;
/// Two metric values closer than this are treated as unchanged.
pub const EPS: f64 = 1e-9;
