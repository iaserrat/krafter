//! Rank files by churn x complexity. Complexity is the sum of per-function
//! cognitive complexity in the file (its current working-tree state).

use crate::cmd::hotspot::model::Hotspot;
use crate::engine::FileAnalysis;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

pub fn rank(
    files: &[FileAnalysis],
    churn: &HashMap<String, usize>,
    branch: &HashSet<String>,
    top: usize,
) -> Vec<Hotspot> {
    let mut rows: Vec<Hotspot> = files
        .iter()
        .map(|f| {
            let complexity: f64 = f.functions.iter().map(|m| m.cognitive).sum();
            let changes = churn.get(&f.path).copied().unwrap_or(0);
            Hotspot {
                path: f.path.clone(),
                changes,
                complexity,
                score: changes as f64 * complexity,
                in_branch: branch.contains(&f.path),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    rows.truncate(top);
    rows
}
