//! One function's quality change across the branch. Missing side is reported as
//! 0.0 (added => before 0; removed => after 0) so the JSON stays flat.

use serde::Serialize;

#[derive(Serialize)]
pub struct FunctionDelta {
    pub path: String,
    pub name: String,
    /// "added" | "removed" | "changed".
    pub status: String,
    pub before_cognitive: f64,
    pub after_cognitive: f64,
    pub delta_cognitive: f64,
    pub before_cyclomatic: f64,
    pub after_cyclomatic: f64,
    pub delta_cyclomatic: f64,
}
