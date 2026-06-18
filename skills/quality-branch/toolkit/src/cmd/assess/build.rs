//! Build one candidate from a paired function. Gate: the function must exist on
//! HEAD and either be new or have a measured metric move (cognitive/cyclomatic/
//! params/exits), and earn at least one reason code. Scope is measured-quality
//! change, by design — a metric-neutral edit (e.g. a pure rename inside the
//! body) is intentionally not surfaced, and a function merely sitting in a
//! changed file is not a finding.

use crate::cmd::assess::model::{Candidate, RepoContext};
use crate::cmd::assess::{reason, score};
use crate::engine::pair::{Change, FnPair};
use crate::engine::FunctionMetrics;
use crate::util::defaults::EPS;
use crate::util::stats::percentile_rank;

pub fn build(p: &FnPair, ctx: &RepoContext) -> Option<Candidate> {
    let after = p.after.as_ref()?;
    let before = p.before.as_ref();
    let bc = before.map(|m| m.cognitive).unwrap_or(0.0);
    let bx = before.map(|m| m.cyclomatic).unwrap_or(0.0);
    let delta_cog = after.cognitive - bc;
    if !touched(p.change, delta_cog, after.cyclomatic - bx, before, after) {
        return None;
    }
    let pct = percentile_rank(ctx.cognitive_sorted, after.cognitive);
    let in_hotspot = ctx.hotspots.contains(&p.path);
    let reasons = reason::codes(p.change, delta_cog, pct, after.params, after.exits, in_hotspot);
    if reasons.is_empty() {
        return None;
    }
    let churn = ctx.churn.get(&p.path).copied().unwrap_or(0);
    Some(Candidate {
        path: p.path.clone(),
        function: p.name.clone(),
        start_line: after.start_line,
        end_line: after.end_line,
        direction: direction(p.change, delta_cog).to_string(),
        reasons,
        before_cognitive: bc,
        after_cognitive: after.cognitive,
        delta_cognitive: delta_cog,
        before_cyclomatic: bx,
        after_cyclomatic: after.cyclomatic,
        delta_cyclomatic: after.cyclomatic - bx,
        cognitive_percentile: pct,
        params: after.params,
        exits: after.exits,
        churn,
        in_hotspot,
        rank: score::rank(pct, delta_cog, churn),
    })
}

fn touched(change: Change, d_cog: f64, d_cyc: f64, before: Option<&FunctionMetrics>, after: &FunctionMetrics) -> bool {
    if change == Change::Added {
        return true;
    }
    let biomarker = before.is_some_and(|b| (b.params - after.params).abs() > EPS || (b.exits - after.exits).abs() > EPS);
    d_cog.abs() > EPS || d_cyc.abs() > EPS || biomarker
}

fn direction(change: Change, delta_cog: f64) -> &'static str {
    match change {
        Change::Added => "new",
        _ if delta_cog > EPS => "regression",
        _ if delta_cog < -EPS => "improved",
        _ => "changed",
    }
}
