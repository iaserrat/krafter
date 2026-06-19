---
name: contract-branch
description: >-
  Evidence-backed, deterministic breaking-change review of a git branch's changes in whatever repository
  the skill is invoked from. Whether a change "breaks callers" is a judgement; this skill makes the
  EVIDENCE deterministic: a bundled Rust tool (ctk), built on tree-sitter, extracts the public contract
  surface (every public function, method, type, field, variant, constant, and alias, with normalized,
  formatter-proof signatures and visibility) across Rust, TypeScript/JavaScript, Python, and Go at the base
  and at the branch head, diffs them, and emits reason-coded candidates — REMOVED, REMOVED_DEPRECATED,
  SIGNATURE_CHANGED, VISIBILITY_REDUCED, ADDED — each with a semver impact and the before/after signature as
  proof, plus an optional CI gate. Every finding cites a signature change at file:line. No measurement, no
  finding. Use whenever the user asks whether a branch/PR breaks the public API or ABI, is
  backward-compatible, needs a major/minor semver bump, changed an exported contract, removed or renamed
  public functions/types/fields, or asks "will this break
  callers", "is this a breaking change", "what semver bump does this need". Produces a ranked,
  signature-cited breaking-change report with a semver call — it assesses, it does not fix.
---

# Contract-Branch

**You measure the contract first, then judge the blast radius.** Whether a change breaks downstream callers
is a judgement — but the _evidence_ you judge from does not have to be. This skill grounds every
compatibility claim in a deterministic signature diff, so the review is reproducible in a PR instead of a
wall of "this might break something".

The bundled Rust tool **`ctk`** computes the facts: the public contract surface at the base ref and at the
branch head, and the reason-coded diff between them. Your job is to turn those into a small set of
high-conviction findings — each anchored to a `before → after` signature at `file:line` — and a single
semver call. **No measurement, no finding** — anything you cannot tie to a `ctk` candidate is a "consider".

This skill is **repo-agnostic**. `ctk` reads a public contract from Rust, TypeScript/JavaScript, Python,
and Go (parsing each with tree-sitter); it reports anything else as unmeasured rather than guessing.

## Core directive

Hold this the entire time:

> Diff the branch's public contract against its base before you judge compatibility. Report only what a
> signature change backs, decide the true blast radius per finding (source-breaking? ABI? cosmetic?), give
> one defensible semver call, and keep it to the changes that matter. The determinism is in the evidence,
> not in the verdict.

## Rules of engagement (the determinism discipline)

1. **No measurement, no finding.** Every finding cites a `ctk` candidate: a reason code + `file:line` + the
   `before`/`after` signature. If you cannot, it is a "consider", filed separately.
2. **Codes, not verdicts.** `SIGNATURE_CHANGED` means the signature moved — _you_ decide whether it is
   genuinely source-breaking (a removed param) or compatible (a param rename for positional callers, a
   widened type). The tool surfaces candidates deterministically; the breaking call is yours.
3. **Reproducible over clever.** Same branch + same base → same findings. `ctk` is deterministic and
   formatter-proof (reformatting, reordering, body rewrites never flag). State the base ref you measured.
4. **Honour the honesty flags.** `ctk` reports `unmeasured` files (a language it has no contract rule for,
   e.g. Go/C/C++) and `parse_ok=false`. Never present "no breaking changes" as a guarantee over files the
   tool could not read — say what was and wasn't measured.
5. **Mind the scope limits.** `ctk` measures functions, methods, types, fields, variants, constants, and
   aliases across the four languages. It does **not** resolve types (a changed param *type* is flagged as
   `SIGNATURE_CHANGED`; you judge source-compatibility), and a few constructs are known gaps — Rust
   trait-*impl* methods, a `pub` member of a private type, Go multi-name const specs, TS `export { x }`
   re-exports. If the diff turns on one of those, inspect by hand and file findings as `CONSIDER`.
6. **No nit floods.** A wall of additive (`ADDED`) entries buries the one `REMOVED`. Lead with the
   breaking set; additive surface is context, not a problem.
7. **Assess; do not fix.** Produce the finding, the signature diff, and a migration pointer. Do not edit
   application code.

## What is deterministic vs what is judgement

- **Deterministic (the tool's job):** what public symbols exist at each ref, their normalized signatures,
  which were removed / changed / narrowed / added, and the semver impact _in isolation_.
- **Judgement (your job):** whether a flagged change actually breaks a real caller here (positional vs
  keyword args, an overload, an internal-only "public" symbol), whether a removal has a deprecation path,
  what migration the caller needs, and the one overall semver call. The determinism is in _what gets
  surfaced_, not in the verdict.

## Phase 0 — Recon: learn the target and what "public" means here

Default the target to **the repo you were invoked from and its current branch**; if the user named another
repo/branch/PR, target that.

```bash
REPO=$(git rev-parse --show-toplevel)
git -C "$REPO" branch --show-current
BASE=$(git -C "$REPO" symbolic-ref --quiet refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
BASE=${BASE:-$(git -C "$REPO" rev-parse --verify -q main >/dev/null && echo main || echo master)}
```

Discover, in roughly this order:

1. **Stack** — languages of the diff (`ctk` reads contracts from Rust, TS/JS, Python, Go; note any the
   diff touches that it cannot, e.g. Go/C/C++ — those need manual review).
2. **What counts as public _here_** — is this a published library/crate/package with external consumers
   (every exported symbol is a contract), an internal service (only the wire/API boundary matters), or an
   app (maybe nothing is a hard contract)? Read `CONTRIBUTING.md`, the package manifest, any
   `#[deprecated]`/`@deprecated` conventions, and whether the project documents a semver policy. This
   decides how much a `REMOVED` actually costs.
3. **Existing API surface** — where the public API lives (a `lib.rs`, an `index.ts`, a package's `__init__`),
   so you can tell a genuine contract from an incidentally-`pub` helper.

Build the toolkit once (needs Rust; see `toolkit/README.md`):

```bash
cd toolkit && make setup        # verifies cargo, builds target/release/ctk
```

Record the target/scope and the "what is public here" call in two or three lines — it becomes the report
header and frames every severity.

## Phase 1 — Scope the diff

```bash
git -C "$REPO" fetch -q origin "$BASE" 2>/dev/null || true
git -C "$REPO" diff --stat "origin/$BASE...HEAD" 2>/dev/null || git -C "$REPO" diff --stat "$BASE...HEAD"
```

If the branch touches no source in a language `ctk` reads a contract from, say so and fall back to a manual
contract review (or stop). Otherwise, note the changed source files and languages.

## Phase 2 — Measure with `ctk` (the deterministic core)

Run from the repo root. `ctk` discovers the repo and base itself; override with `--repo` / `--base`.

**`assess` is the spine.** It diffs the branch's public contract against the base and emits the ranked,
reason-coded candidates plus the overall semver impact:

```bash
toolkit/target/release/ctk --base "$BASE" assess
```

Read the result top to bottom:

- **`semver_impact`** — the headline (`major` / `minor` / `none`). This is your starting semver call; you
  confirm or downgrade it with judgement.
- **`candidates`** — each is an evidence bundle: `path`, `symbol` (qualified, e.g. `Config::port` for a
  field or `Service::handle` for a method), `kind`, `line`, `reason`, `semver`, `breaking`, and the
  `before`/`after` signatures. Reason codes map to the break catalog:
  - `REMOVED` — a public symbol is gone (or the file was deleted). The clearest break.
  - `REMOVED_DEPRECATED` — a removed symbol that was already `#[deprecated]`/`@deprecated` at base; still
    breaking, but it had a migration window, so it sorts below fresh removals.
  - `SIGNATURE_CHANGED` — same public symbol, the normalized signature moved. Read `before`/`after` to see
    what (a param added/removed/retyped, a field's type changed, the return changed, a rename).
  - `VISIBILITY_REDUCED` — `pub` → private and similar. Breaks any caller that reached it.
  - `ADDED` — new public surface (a function, type, field, variant, const). Additive; credit it.
- **`gate`** — `{ impact, tripped, suppressed }`. `tripped` is only ever true under `--fail-on`; otherwise
  the run is report-only. `suppressed` counts candidates excluded by a `--baseline`.
- **`unmeasured`** — changed files the tool could not read a contract from (an unsupported language).
  Review these by hand; they are coverage gaps, not clean bills of health.

For a CI gate, add `--fail-on major` (exit 2 on any breaking change) or `--fail-on minor` (any contract
change), and `--baseline <file>` to accept known breaks (`REASON path symbol` per line). The headline
`semver_impact` always reflects the unfiltered truth even when the gate is suppressed.

Drill into a specific file's contract on either side with the building block when you need to see the full
surface (e.g. to confirm an incidentally-`pub` helper vs a real API symbol):

```bash
toolkit/target/release/ctk surface --paths path/to/api.rs   # working-tree (head) surface
git show "$BASE:path/to/api.rs" > /tmp/base.rs && ctk surface --paths /tmp/base.rs   # base surface
```

## Phase 3 — Judge: turn candidates into findings and one semver call

`assess` surfaced and ranked the candidates. Your job is the judgement the tool can't make:

- For each **breaking** candidate, decide whether it breaks a _real_ caller here. A `SIGNATURE_CHANGED` that
  only renamed a positional parameter is cosmetic in Rust/Go but breaking in Python (keyword args). A
  `REMOVED` on a symbol that is `pub` only for tests is not a real contract break. A widened parameter type
  may be source-compatible. Downgrade or confirm with a reason.
- For each confirmed break, write the **migration**: what the caller must change, and whether a deprecation
  shim is possible instead of a hard removal.
- Make **one overall semver call.** Start from `semver_impact`; downgrade it only with an explicit
  justification per candidate ("the only `REMOVED` is a test-only symbol"). If anything genuinely breaks a
  real caller, it is a major.

A candidate becomes a finding only with its signature diff attached. Use `references/break-catalog.md` for
the taxonomy (each reason code → what to check → typical migration).

## Phase 4 — Report

Write to `contract-<branch>-<date>.md` in the repo root (offer the path; do not commit). Follow
`references/report-template.md`. Lead with the semver call. Every finding carries:

- **The measurement** — the `ctk` candidate: reason code, `before → after` signature, `file:line`. The spine.
- **The judgement** — does this break a real caller here, and why (2-3 sentences).
- **Migration** — what a caller changes, or the deprecation path instead of a hard break.
- **Confidence** — `MEASURED` (ctk-backed) vs `CONSIDER` (judgement-only: an unmeasured-language file, or
  one of the known gaps — a Rust trait-impl method, a `pub` member of a private type) — keep these visibly
  separate.

**End with a clear semver verdict and a migration punch list.** State the bump (major/minor/none) and why
in terms of the candidates, then the ordered list: what breaks callers (the confirmed `REMOVED` /
`SIGNATURE_CHANGED` / `VISIBILITY_REDUCED`), what is merely additive, and what is unmeasured and needs a
manual look. If the contract is unchanged, say "no breaking changes, no bump needed" and stop.

## Calibrate to the ask

Scale to the request. "Did this branch break the API?" → `assess`, confirm the breaking candidates, give the
semver call. "Full backward-compat audit" / "be strict" → `assess` plus `surface` on every changed public
file, manually cover the scope gaps (unmeasured languages and the known type-resolution gaps), and
write the full report with migrations. For a merge gate, wire `assess --fail-on major` into CI. When
unsure, lean thorough — but never inflate the finding count past what the candidates support, and never
present an unmeasured file as a clean bill of health.
