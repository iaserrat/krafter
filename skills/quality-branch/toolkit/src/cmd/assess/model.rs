//! The assess evidence bundle. Every raw measurement is reported separately
//! (never blended into one opaque score), so an LLM reviewer can cite each one
//! at file:line. The rank is transparent: formula + per-term values are shown.

use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// Repo-wide context a candidate is scored against (borrowed, not serialized).
pub struct RepoContext<'a> {
    pub cognitive_sorted: &'a [f64],
    pub churn: &'a HashMap<String, usize>,
    pub hotspots: &'a HashSet<String>,
}

/// The deterministic, auditable rank: score plus the exact terms that made it.
#[derive(Serialize)]
pub struct RankInfo {
    pub score: f64,
    pub formula: String,
    pub percentile_term: f64,
    pub delta_term: f64,
    pub churn_term: f64,
}

/// One ranked candidate finding: anchor + every triggering raw measurement +
/// reason codes + direction. The skill turns this into a cited finding.
#[derive(Serialize)]
pub struct Candidate {
    pub path: String,
    pub function: String,
    pub start_line: usize,
    pub end_line: usize,
    /// "new" | "regression" | "improved" | "changed".
    pub direction: String,
    pub reasons: Vec<String>,
    pub before_cognitive: f64,
    pub after_cognitive: f64,
    pub delta_cognitive: f64,
    pub before_cyclomatic: f64,
    pub after_cyclomatic: f64,
    pub delta_cyclomatic: f64,
    /// Percentile of after_cognitive within this repo's distribution.
    pub cognitive_percentile: f64,
    pub params: f64,
    pub exits: f64,
    pub churn: usize,
    pub in_hotspot: bool,
    pub rank: RankInfo,
}
