//! Transparent ranking. Not a hidden blended grade — the formula and every
//! term are emitted so the ordering is auditable and reproducible. Regressions
//! and high-percentile complexity in churned files rise to the top.

use crate::cmd::assess::model::RankInfo;

const FORMULA: &str = "percentile * (1 + max(0, delta_cognitive)) * (1 + churn)";

pub fn rank(pct: f64, delta_cog: f64, churn: usize) -> RankInfo {
    let percentile_term = pct;
    let delta_term = 1.0 + delta_cog.max(0.0);
    let churn_term = 1.0 + churn as f64;
    RankInfo {
        score: percentile_term * delta_term * churn_term,
        formula: FORMULA.to_string(),
        percentile_term,
        delta_term,
        churn_term,
    }
}
