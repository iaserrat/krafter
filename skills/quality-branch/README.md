# quality-branch

A **deterministic-evidence** code-quality skill for Claude Code: measure what a branch's diff did to the
code with a bundled Rust tool, then judge from the numbers. Quality is subjective, so the skill makes the
*evidence* reproducible — every finding must cite a measurement at `file:line`. Repo-, stack-, and
language-agnostic; it learns the target and reuses the repo's own linters and conventions.

It produces a severity-ranked, measurement-cited findings report with remediation pointers. It assesses;
it does not fix.

## Why this exists

LLM code-quality review drifts run-to-run and is hard to defend in a PR ("this feels over-engineered").
This skill borrows the structure of `red-team-branch`: the LLM does judgement, a deterministic Rust tool
(`cqt`) provides the evidence. The discipline — *no measurement, no finding* — kills hallucinated and
inconsistent findings and makes the review reproducible.

It deliberately does **not** rank by absolute thresholds. Cyclomatic complexity correlates heavily with
raw size and explains only a fraction of defect variance, so absolute cutoffs are folklore. `cqt` ranks by
**branch deltas** (before→after) and **repo-relative percentiles**, both reproducible and defensible.

## Layout

| Path | What it is |
|---|---|
| `SKILL.md` | the methodology Claude follows (recon → scope → measure → judge → report) |
| `references/smell-catalog.md` | quality smell classes, each mapped to the `cqt` signal that surfaces it |
| `references/report-template.md` | report skeleton + repo-relative severity rubric |
| `toolkit/` | `cqt`, a self-contained Rust CLI that does the deterministic measuring |
| `toolkit/README.md` | `cqt` command reference + worked examples |

## Setup

The skill itself needs no build — Claude reads `SKILL.md` and the references directly. The **toolkit**
needs a one-time build.

### No prebuilt binary

This repo ships **no compiled `cqt` binary** — build from source you can audit (`toolkit/target/` is
gitignored). The metrics engine is a pinned commit of Mozilla's tree-sitter-based `rust-code-analysis`
(the crates.io release is stale; master is maintained, so we pin a reviewed SHA).

### Prerequisites

- **Rust toolchain** (cargo + rustc) via [rustup](https://rustup.rs). A C compiler is needed once to build
  the tree-sitter grammars.

### Build

```bash
cd toolkit
make setup        # verifies the toolchain, builds target/release/cqt
```

## Using the skill

Ask Claude to review a branch's quality (e.g. *"review this branch for code quality"*, *"did this branch
make the code more complex?"*, *"where's the risky code in this PR?"*). The skill will:

1. **Recon** — learn the stack and run the repo's own linters/conventions.
2. **Scope** — pull the diff; note the changed source files.
3. **Measure** — `cqt assess` joins the signals into ranked, reason-coded candidate findings (delta ×
   repo-percentile × hotspot × biomarkers); `cqt dup` and `cqt coupling` add branch-introduced duplication
   and missing co-change. Drill in with `calibrate`/`delta`/`hotspot`/`metrics`.
4. **Judge** — decide which measured outliers are real problems here; write the fix.
5. **Report** — ranked, measurement-cited findings with remediation pointers, closing with a clear,
   actionable summary (merge-readiness call + prioritized punch list).

See `SKILL.md` for the methodology and `toolkit/README.md` for the `cqt` reference.

## Relationship to other skills

`cqt` provides the deterministic measurement layer that judgement-heavy reviewers lack. Pair it with a
strict maintainability reviewer (e.g. a "thermo-nuclear" code-quality skill) for qualitative restructuring
ideas — `cqt` tells you *where* to point that review; the reviewer supplies the "code judo".
