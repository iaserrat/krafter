//! The spine: diff the branch's public contract against its base and emit
//! ranked, reason-coded breaking-change candidates with an overall semver
//! impact. Codes, not verdicts — the skill decides what is truly breaking.
//! Report-only by default; `--fail-on` turns it into a CI gate (exit code).

pub mod args;
pub mod classify;
pub mod gate;
pub mod load;
pub mod model;

pub use args::Args;

use crate::cmd::{self, Ctx};
use model::{MAJOR, MINOR, NONE};
use serde_json::json;
use std::collections::BTreeSet;

pub fn run(a: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let mut loaded = load::load(ctx)?;
    let baseline = match &a.baseline {
        Some(p) => gate::load_baseline(p)?,
        None => BTreeSet::new(),
    };
    gate::apply_baseline(&mut loaded.candidates, &baseline);

    let impact = semver_impact(&loaded.candidates);
    let gate_impact = gate::effective_impact(&loaded.candidates);
    let code = gate::exit_code(a.fail_on, gate_impact);
    let suppressed = loaded.candidates.iter().filter(|c| c.suppressed).count();

    cmd::emit(&json!({
        "command": "assess",
        "schema": "ctk.assess/v2",
        "base": ctx.base,
        "semver_impact": impact,
        "gate": { "impact": gate_impact, "tripped": code != 0, "suppressed": suppressed },
        "candidates": loaded.candidates,
        "unmeasured": loaded.unmeasured,
    }));

    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// The headline over *all* candidates (suppression only affects the gate, not the
/// reported impact): `major` if any change can break a caller, else `minor` if
/// the branch only added public surface, else `none`.
fn semver_impact(candidates: &[model::Candidate]) -> &'static str {
    if candidates.iter().any(|c| c.breaking) {
        MAJOR
    } else if candidates.is_empty() {
        NONE
    } else {
        MINOR
    }
}
