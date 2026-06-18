//! Deterministic percentile statistics. Repo-relative percentiles are how the
//! skill avoids arbitrary absolute thresholds: "p99 complexity here", not
//! "complexity > 10".

use crate::util::defaults::{P50, P75, P90, P95, P99, PCT_MAX};
use serde::Serialize;

/// Percentile points of one metric across the repo, plus count and max.
#[derive(Serialize)]
pub struct Distribution {
    pub count: usize,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

/// Nearest-rank percentile over an ascending-sorted slice (deterministic).
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p / PCT_MAX * sorted.len() as f64).ceil() as usize;
    let idx = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}

/// Percentile RANK of `value` within an ascending-sorted slice: the share of
/// values <= value, in [0,100]. The repo-relative position of a measurement.
pub fn percentile_rank(sorted: &[f64], value: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let le = sorted.iter().filter(|&&x| x <= value).count();
    le as f64 / sorted.len() as f64 * PCT_MAX
}

/// Build the distribution from unsorted values.
pub fn distribution(mut values: Vec<f64>) -> Distribution {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Distribution {
        count: values.len(),
        p50: percentile(&values, P50),
        p75: percentile(&values, P75),
        p90: percentile(&values, P90),
        p95: percentile(&values, P95),
        p99: percentile(&values, P99),
        max: values.last().copied().unwrap_or(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_track_position() {
        let d = distribution((1..=100).map(|n| n as f64).collect());
        assert_eq!(d.count, 100);
        assert_eq!(d.max, 100.0);
        assert_eq!(d.p50, 50.0);
        assert_eq!(d.p99, 99.0);
    }

    #[test]
    fn rank_is_relative_position() {
        let sorted: Vec<f64> = (1..=100).map(|n| n as f64).collect();
        assert_eq!(percentile_rank(&sorted, 50.0), 50.0);
        assert_eq!(percentile_rank(&sorted, 100.0), 100.0);
        assert_eq!(percentile_rank(&[], 5.0), 0.0);
    }
}
