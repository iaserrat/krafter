use serde::Serialize;

/// One file's hotspot row. `score = changes * complexity`; `in_branch` marks
/// files the current branch modifies.
#[derive(Serialize)]
pub struct Hotspot {
    pub path: String,
    pub changes: usize,
    pub complexity: f64,
    pub score: f64,
    pub in_branch: bool,
}
