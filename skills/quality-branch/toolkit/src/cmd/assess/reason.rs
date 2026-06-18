//! Reason codes, not verdicts. The tool labels the evidence; the LLM writes the
//! prose. Complexity thresholds are repo-relative percentiles (P90/P95);
//! biomarker thresholds (args/exits) are absolute by design — a 7-arg function
//! is hard to use regardless of the repo.

use crate::engine::pair::Change;
use crate::util::defaults::{EPS, P90, P95};

/// A function with at least this many parameters is flagged (Code Climate uses 4).
const NARGS_FLAG: f64 = 5.0;
/// A function with at least this many exit points is flagged.
const NEXITS_FLAG: f64 = 4.0;

pub fn codes(change: Change, delta_cog: f64, pct: f64, params: f64, exits: f64, in_hotspot: bool) -> Vec<String> {
    let mut r: Vec<String> = Vec::new();
    match change {
        Change::Added => {
            r.push("NEW_FUNCTION".into());
            if pct >= P90 {
                r.push("NEW_COMPLEXITY_HIGH".into());
            }
            if in_hotspot {
                r.push("NEW_IN_HOTSPOT".into());
            }
        }
        Change::Changed => {
            if delta_cog > EPS {
                r.push("COMPLEXITY_REGRESSION".into());
                if pct >= P95 {
                    r.push("REGRESSION_ABOVE_P95".into());
                }
                if in_hotspot {
                    r.push("COMPLEXITY_IN_HOTSPOT".into());
                }
            }
            if delta_cog < -EPS {
                r.push("IMPROVED".into());
            }
        }
        Change::Removed => {}
    }
    if params >= NARGS_FLAG {
        r.push("LARGE_ARG_LIST".into());
    }
    if exits >= NEXITS_FLAG {
        r.push("MANY_EXIT_POINTS".into());
    }
    r
}
