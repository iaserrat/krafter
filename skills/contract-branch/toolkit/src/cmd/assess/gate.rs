//! CI gate: mark baselined candidates as suppressed and decide the process exit
//! code from the contract change. Report-only by default — a gate is opt-in via
//! `--fail-on`, so existing agent-driven use never changes behaviour.

use crate::cmd::assess::args::FailOn;
use crate::cmd::assess::model::{Candidate, MAJOR, MINOR, NONE};
use std::collections::BTreeSet;
use std::path::Path;

/// Exit code when the gate trips. Distinct from 1 so a real error (bad repo) and
/// a tripped gate are tell-apart in CI.
pub const EXIT_GATE: i32 = 2;

/// Read baseline keys: one `REASON path symbol` per line, `#` comments allowed.
/// A missing/unreadable file is a hard error — silently ignoring it would let a
/// stale baseline hide real breaks.
pub fn load_baseline(path: &Path) -> anyhow::Result<BTreeSet<String>> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("baseline {}: {e}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect())
}

/// Stable key identifying a candidate in a baseline file.
pub fn key(c: &Candidate) -> String {
    format!("{} {} {}", c.reason.code(), c.path, c.symbol)
}

/// Mark candidates whose key is in the baseline as suppressed (excluded from the
/// gate decision but kept in the JSON for transparency).
pub fn apply_baseline(candidates: &mut [Candidate], baseline: &BTreeSet<String>) {
    for c in candidates.iter_mut() {
        c.suppressed = baseline.contains(&key(c));
    }
}

/// Impact of the non-suppressed candidates (what the gate judges).
pub fn effective_impact(candidates: &[Candidate]) -> &'static str {
    let live = || candidates.iter().filter(|c| !c.suppressed);
    if live().any(|c| c.breaking) {
        MAJOR
    } else if live().next().is_some() {
        MINOR
    } else {
        NONE
    }
}

/// The exit code: `EXIT_GATE` if the effective impact meets the threshold, else 0.
pub fn exit_code(fail_on: Option<FailOn>, impact: &str) -> i32 {
    let trips = match fail_on {
        None => false,
        Some(FailOn::Major) => impact == MAJOR,
        Some(FailOn::Minor) | Some(FailOn::Any) => impact != NONE,
    };
    if trips {
        EXIT_GATE
    } else {
        0
    }
}
