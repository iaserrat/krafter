---
name: quality-branch
description: >-
  Evidence-backed, deterministic code-quality review of a git branch's changes in whatever repository
  the skill is invoked from — language and framework agnostic. Quality judgement is subjective, so this
  skill makes the EVIDENCE deterministic: a bundled Rust tool (cqt) measures per-function complexity
  deltas (cognitive/cyclomatic), repo-relative percentiles, churn-x-complexity hotspots, branch-introduced
  duplication, and missing co-change coupling, and joins them with `cqt assess` into ranked, reason-coded
  candidate findings. Every finding must cite a measurement at file:line. No measurement, no finding. Use
  whenever the user asks to review code quality, maintainability, complexity, or duplication of a
  branch/PR/diff, asks "did this branch make the code worse", "what got more complex", "where's the risky
  code", or wants a reproducible, metric-grounded quality assessment rather than vibes. Produces a ranked,
  measurement-cited findings report with remediation pointers — it assesses, it does not fix.
---

# Quality-Branch

**You measure first, then judge.** Quality is subjective — but the _evidence_ you judge from does not have
to be. This skill's whole point is to ground every quality claim in a deterministic measurement, so the
review is reproducible across runs and defensible in a PR, instead of a wall of unfalsifiable opinions.

The bundled Rust tool **`cqt`** computes the numbers: per-function complexity deltas the branch
introduced, the repo's own complexity distribution, and churn × complexity hotspots. Your job is to turn
those numbers into a small set of high-conviction findings, each one anchored to a measurement at
`file:line`, with a concrete remediation. **No measurement, no finding** — anything you cannot tie to a
`cqt` number is a "consider", not a finding.

This skill is **repo-agnostic and stack-agnostic**. It assumes a _methodology_, not a framework — and it
reuses the repo's own linters and conventions rather than reinventing them.

## Core directive

Hold this the entire time:

> Measure what the branch did to the code before you judge it. Rank by the repo's own distribution and by
> the branch's deltas, never by absolute folklore thresholds. Report only what a number backs, point each
> finding at a concrete fix, and keep it to the few that matter. The determinism is in the evidence, not
> in the verdict.

## Rules of engagement (the determinism discipline)

These are what separate this from a prose review with opinions.

1. **No measurement, no finding.** Every finding cites a `cqt` measurement (a metric value + `file:line`,
   a delta, a percentile rank, or a hotspot score). If you cannot, it is a "consider", filed separately.
2. **Rank by delta and percentile, never absolutes.** "This branch pushed `processPayment` from cognitive
   12 → 41" and "p99 complexity for this repo" are reproducible. "Complexity > 10 is bad" is folklore —
   cyclomatic correlates heavily with raw size and explains only a fraction of defect variance. Use
   `cqt calibrate` so severity is relative to _this_ codebase.
3. **Reproducible over clever.** Same branch + same base → same findings. The tool is deterministic (no
   wall-clock, no sampling); your ranking must be too. State the base ref you measured against.
4. **Honour the parse-honesty flag.** `cqt` reports `parse_ok=false` / `parse_errors` when a file's
   language is unsupported or parsing was lossy. Downgrade or omit findings on those files — never present
   a number the tool flagged as unreliable.
5. **Reuse the repo's own bar.** Run the project's existing linters/formatters and read their configs
   first (Phase 0). A `cqt` complexity delta is _additional_ signal, not a replacement for the team's
   established rules.
6. **No nit floods.** A wall of "this is a bit long" buries the one function that doubled in cognitive
   complexity. High-conviction, measurement-backed findings only.
7. **Assess; do not fix.** Produce the finding, the measurement, and a remediation pointer. Do not edit
   application code.

## What is deterministic vs what is judgement (be honest about the boundary)

- **Deterministic (the tool's job):** complexity values and their deltas, where complexity concentrated,
  which changed functions are repo-relative outliers, which touched files are churn × complexity hotspots.
- **Judgement (your job):** whether a measured outlier is _actually_ a problem here, whether an
  abstraction earns its keep, naming, cohesion, whether a duplication is the bad kind. The tool surfaces
  **candidates** deterministically; you decide which are real and how to fix them. The determinism is in
  _what gets surfaced and measured_, not in the verdict.

## Phase 0 — Recon: learn the target and its own quality bar

Default the target to **the repo you were invoked from and its current branch**; if the user named another
repo/branch/PR, target that.

```bash
REPO=$(git rev-parse --show-toplevel)
git -C "$REPO" branch --show-current
BASE=$(git -C "$REPO" symbolic-ref --quiet refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
BASE=${BASE:-$(git -C "$REPO" rev-parse --verify -q main >/dev/null && echo main || echo master)}
```

Discover, in roughly this order:

1. **Stack** — languages/frameworks (dependency manifests, layout). `cqt` measures Rust, Python, JS/TS,
   C/C++, Go, Java and more; note which languages the diff actually touches.
2. **The repo's own quality bar = your reference.** Hunt for linter/formatter configs (`clippy.toml`,
   `.eslintrc`, `ruff.toml`, `.editorconfig`), `CONTRIBUTING.md`, `CLAUDE.md`/`AGENTS.md`, complexity
   gates in CI, and **any companion skill installed for this repo**. Run the project's linters and read
   their output — that is the team's established bar, and your remediation vocabulary. Do not reinvent it.
3. **How code is structured here** — the dominant patterns (where helpers live, how modules are split,
   typical function size). Your findings reference _this_ repo's conventions, not generic best practice.

Build the toolkit once (needs Rust; see `toolkit/README.md`):

```bash
test -x toolkit/target/release/cqt || (cd toolkit && make setup)   # idempotent: skips the rebuild if already built
```

Record the target/scope in two or three lines — it becomes the report header.

## Phase 1 — Scope the diff

```bash
git -C "$REPO" fetch -q origin "$BASE" 2>/dev/null || true
git -C "$REPO" diff --stat "origin/$BASE...HEAD" 2>/dev/null || git -C "$REPO" diff --stat "$BASE...HEAD"
```

If the branch touches only docs/config/tests with no measurable source, say so and stop — there is nothing
for `cqt` to measure. Otherwise, note the changed source files and which languages they are in.

## Phase 2 — Measure with `cqt` (the deterministic core)

Run from the repo root. `cqt` discovers the repo and base itself; override with `--repo` / `--base`. Read
the parse-honesty signal (`parse_ok` / `parse_errors` / skipped) on every result — a file flagged
unreliable does not get a finding.

**Start with `assess` — it is the spine.** It joins the four complexity signals into one ranked list of
reason-coded candidate findings, so you do not have to cross-reference by hand:

```bash
toolkit/target/release/cqt --base "$BASE" assess
```

Each candidate is a self-contained evidence bundle: an anchor (`file:line`, function), every raw
measurement separately (before→after cognitive/cyclomatic, the repo-relative `cognitive_percentile`,
`params`/`exits`, `churn`, `in_hotspot`), a transparent `rank` (formula + per-term values), `reasons`
(codes, not verdicts), and a `direction` (`new`/`regression`/`improved`/`changed`). Work the top of the
list. The reason codes map to the smell catalog:

- `REGRESSION_ABOVE_P95` / `NEW_COMPLEXITY_HIGH` — a repo-relative complexity outlier the branch caused.
- `COMPLEXITY_IN_HOTSPOT` / `NEW_IN_HOTSPOT` — complexity landing on already-fragile, frequently-changed code.
- `LARGE_ARG_LIST` / `MANY_EXIT_POINTS` — structural biomarkers a single complexity number misses.
- `IMPROVED` — a cleanup; credit it.

Then run the **diff-scoped** duplication and coupling checks (both default to branch scope):

```bash
toolkit/target/release/cqt --base "$BASE" dup       # clones the branch introduced (incl. copies of existing code)
toolkit/target/release/cqt --base "$BASE" coupling  # "you changed A but not B, which usually co-changes" (status: missed)
```

Use the underlying single-signal commands when you need to drill in or want the raw distribution:
`calibrate` (the repo's percentile distribution behind `assess`), `delta` (before→after per function),
`hotspot --top 20` (churn × complexity ranking), `metrics --paths <file>` (per-function numbers, no git).

## Phase 3 — Judge: turn measurements into findings

`assess` already surfaced and ranked the candidates. Your job is judgement an LLM is good at and the tool
is not: for each top candidate, read the function and decide whether the measured outlier is _actually_ a
maintainability problem here. Is the control flow genuinely tangled, is there a missing abstraction, a
repeated conditional that wants a model, a wrapper that adds indirection without clarity, a `dup` clone
that should be extracted, a `coupling` `missed` partner that signals a forgotten edit or a broken
invariant?

A candidate becomes a finding only with its measurement attached. Write a concrete remediation pointing at
an existing pattern in _this_ repo. If the repo has a companion review skill (e.g. a strict maintainability
reviewer), use it for the qualitative restructuring ideas — `cqt` tells you _where_ to point it, and the
reason codes tell you _what kind_ of problem to look for.

## Phase 4 — Report

Write to `quality-<branch>-<date>.md` in the repo root (offer the path; do not commit). Follow
`references/report-template.md`. Every finding carries:

- **The measurement** — the `cqt` number(s): before→after, the delta, the percentile rank, the hotspot
  score, with `file:line`. This is the spine of the finding.
- **The judgement** — why this measured outlier is actually a maintainability problem here (2-3 sentences).
- **Recommendation** — the concrete restructuring, pointing at an existing safe pattern in this repo.
- **Confidence** — `MEASURED` (number-backed) vs `CONSIDER` (judgement-only, no metric) — keep these
  visibly separate so the reader trusts the measured set.

**End the report with a clear, actionable summary.** After the ranked findings, close with a short
"Summary (clear, actionable)" section that synthesizes the report into a punch list for the engineer
reviewing the branch. Keep the metric vocabulary — the audience is technical. Lead with merge-readiness
(good to merge / merge-with-followups / needs-changes-first, and why, in terms of the measured deltas),
then give a prioritized action list: what to fix before merging (the High findings), what's optional
cleanup, what's already fine — each action naming the function/file and the concrete move. Keep measured
obligations separate from `CONSIDER` opinion, and if the branch is clean, say "merge it" and stop. See the
template's `Summary` section.

Use `references/smell-catalog.md` for the vocabulary of quality smell classes and which `cqt` signal
surfaces each.

### Then open it in the browser

After writing the Markdown report, render it as a single self-contained **HTML** file and open it so the
reader sees the findings immediately — same content, presented for reading:

- **Minimal editorial style.** One readable column (`max-w-3xl mx-auto`, generous vertical rhythm), a
  clear type hierarchy, a restrained palette — a well-typeset article, not a dashboard. Use **Tailwind**
  via the Play CDN (`<script src="https://cdn.tailwindcss.com"></script>`); monospace for `file:line`,
  measurements, and code; reserve color for the merge-readiness verdict and the `MEASURED`/`CONSIDER`
  distinction only. No cards, no chrome, no logos.
- **Save to `/tmp` and overwrite on rerun.** Write to the stable path `/tmp/quality-<branch>.html`
  (sanitize any `/` in the branch name to `-`); overwrite it if it exists, so a rerun refreshes the same
  file instead of piling up renders.
- **Open it:** `open /tmp/quality-<branch>.html` (macOS) / `xdg-open` (Linux).

The Markdown report in the repo root stays the source of record; the HTML is the read-only view.

## Calibrate to the ask

Scale to the request. "Quick quality check of this small branch" → `calibrate` + `delta`, a handful of
measured findings, skip hotspot unless the diff touches core files. "Thorough maintainability audit" /
"be strict" → full `calibrate` + `delta` + `hotspot`, drill into every repo-relative outlier with
`metrics`, hand the qualitative restructuring to a strict-reviewer companion skill, and write the full
report. When unsure, lean thorough — but never inflate the finding count past what the numbers support.
