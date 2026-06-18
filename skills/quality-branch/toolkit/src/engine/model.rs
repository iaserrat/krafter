//! The deterministic measurement the whole skill is built on. Every field is
//! computed by rust-code-analysis from the AST — no judgement, no wall-clock.

use serde::Serialize;

/// One named function/method space and its metrics. `params`/`exits` are
/// structural biomarkers (argument count, exit-point count) that catch what a
/// single complexity number misses.
#[derive(Serialize, Clone)]
pub struct FunctionMetrics {
    /// Qualified name (e.g. `Impl::method`), so deltas match the right pair.
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
    pub cyclomatic: f64,
    pub cognitive: f64,
    pub sloc: f64,
    pub params: f64,
    pub exits: f64,
}

/// One source file's analysis. `parse_ok=false` is the honesty flag: the
/// language was unsupported or parsing was lossy, so callers must downgrade.
#[derive(Serialize, Clone)]
pub struct FileAnalysis {
    pub path: String,
    pub lang: String,
    pub parse_ok: bool,
    pub functions: Vec<FunctionMetrics>,
}
