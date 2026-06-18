//! Map a paired function (base vs HEAD) to a FunctionDelta, dropping pairs
//! whose complexity did not actually move (renamed-but-unchanged, formatting).

use crate::cmd::delta::model::FunctionDelta;
use crate::engine::pair::{Change, FnPair};
use crate::util::defaults::EPS;

const ADDED: &str = "added";
const REMOVED: &str = "removed";
const CHANGED: &str = "changed";

pub fn to_delta(p: &FnPair) -> Option<FunctionDelta> {
    let bc = p.before.as_ref().map(|m| m.cognitive).unwrap_or(0.0);
    let ac = p.after.as_ref().map(|m| m.cognitive).unwrap_or(0.0);
    let bx = p.before.as_ref().map(|m| m.cyclomatic).unwrap_or(0.0);
    let ax = p.after.as_ref().map(|m| m.cyclomatic).unwrap_or(0.0);
    let status = match p.change {
        Change::Added => ADDED,
        Change::Removed => REMOVED,
        Change::Changed => CHANGED,
    };
    if p.change == Change::Changed && (bc - ac).abs() < EPS && (bx - ax).abs() < EPS {
        return None; // unchanged complexity — not a finding
    }
    Some(FunctionDelta {
        path: p.path.clone(),
        name: p.name.clone(),
        status: status.to_string(),
        before_cognitive: bc,
        after_cognitive: ac,
        delta_cognitive: ac - bc,
        before_cyclomatic: bx,
        after_cyclomatic: ax,
        delta_cyclomatic: ax - bx,
    })
}
