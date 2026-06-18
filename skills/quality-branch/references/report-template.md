# Quality-Branch Report Template

Write to `quality-<branch>-<date>.md` in the repo root. Lead with the measured summary, then findings
ranked by severity, and **close with a clear, actionable summary** that synthesizes the findings into a
merge-readiness call and a prioritized punch list. Keep `MEASURED` and `CONSIDER` findings visibly
separate — the reader's trust in the measured set depends on not mixing in opinion.

---

## Target / Scope

- **Repo / branch / base:** `<repo>` · `<branch>` vs `<base>` (state the exact base ref measured).
- **Stack measured:** languages `cqt` analyzed; note any the diff touched that `cqt` could not parse.
- **Repo's own bar:** linters/configs found and run in Phase 0 (what the team already enforces).
- **Diff size:** files changed / insertions / deletions.

## Repo calibration (from `cqt calibrate`)

| Metric | p50 | p75 | p90 | p95 | p99 | max |
|---|---|---|---|---|---|---|
| cognitive | | | | | | |
| cyclomatic | | | | | | |

> These percentiles are the "outlier for this repo" lines every finding below is ranked against. No
> absolute thresholds are used.

## Verdict

One line: **Clean** / **Minor cleanups** / **Notable regressions** / **Significant maintainability regression**.
State it as what the measurements show, e.g. "the branch added two functions above this repo's p99 cognitive
complexity and piled onto its top hotspot."

## Measured findings (ranked, worst first)

Source these from `cqt assess` candidates (already ranked, reason-coded, with the rank formula and
per-term values). For each:

### <n>. <short title> — <severity>

- **Measurement:** the candidate's evidence bundle. e.g. `processPayment` cognitive 12 → 41 (Δ+29), now
  p99 for this repo; cyclomatic 8 → 22; `in_hotspot=true`, churn 47; reasons
  `[REGRESSION_ABOVE_P95, COMPLEXITY_IN_HOTSPOT]`; direction `regression`. Cite `file:line`.
- **Why it's a problem here:** 2-3 sentences of judgement — the actual maintainability cost in this
  codebase (tangled control flow, missing abstraction, repeated conditional, etc.).
- **Recommendation:** the concrete restructuring, pointing at an existing pattern in this repo
  (`file:line`) or the team's convention from Phase 0. Specific, not generic.
- **Smell class:** from `smell-catalog.md` (the assess reason code maps to it) — so the team learns the
  pattern, not just the instance.

## Duplication & coupling (diff-scoped)

- **New clones** (`cqt dup`): for each clone group a branch file participates in, list the members
  (`path:start-end`), token length, and whether it copies pre-existing repo code. Judge whether extraction
  is warranted (smell class 8).
- **Missing co-change** (`cqt coupling`, `status: "missed"`): for each high-`degree` missed pair, name the
  branch-side `anchor` and the untouched partner, and ask whether the partner needs the same change or an
  invariant broke (smell class 9). Omit if there is no compelling co-change history.

## Consider (judgement-only, no measurement)

Findings you believe in but cannot tie to a `cqt` number (naming, an abstraction that feels unwarranted,
a structural idea). Clearly separated so the reader knows these are opinion, not measured fact. Keep short.

## What was measured but held

Brief: functions/files that changed but stayed within the repo's distribution, and any files `cqt` could
not parse (so the reader knows coverage). "No measured regressions" is a statement about *what was
measured*, not a guarantee.

## Severity rubric (repo-relative, not absolute)

- **High** — a changed/new function at or above repo p99, OR a large delta landing on a top hotspot
  (`in_branch`), OR several measured regressions compounding in one module.
- **Medium** — a function crossing into p90–p95 that the branch introduced or worsened materially.
- **Low** — a measurable but modest increase below p90; note it, do not flood with it.
- **Consider** — no measurement; judgement only.

## Summary (clear, actionable)

The closing section: a concise, technical wrap-up that tells the reviewer what to do, in priority order.
Written for engineers — keep the metric vocabulary, skip the hand-holding. No new findings; this is the
synthesis of everything above into a punch list. Keep it to ~5–8 sentences or a few bullets.

- **Lead with merge-readiness.** One line: is the branch good to merge, merge-with-followups, or
  needs-changes-first — and why, in terms of the measured deltas.
- **Give a prioritized action list.** What to address before merging (the High findings), what's optional
  cleanup (Low / `CONSIDER`), and what's already fine. Point each action at its finding above.
- **Be specific and concrete.** Name the function/file and the move (extract X, collapse the repeated
  conditional, split the hotspot), not "reduce complexity."
- **Keep the measured/consider line visible.** Actions backed by a `cqt` number are obligations;
  `CONSIDER` items are optional judgement calls — don't blur them.
- **Stay honest.** If the branch is clean, say "nothing to do, merge it" and stop. Don't manufacture work.
