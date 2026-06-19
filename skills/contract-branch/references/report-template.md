# Contract-Branch Report Template

Write to `contract-<branch>-<date>.md` in the repo root. **Lead with the semver call**, then the breaking
findings, then additive surface, and **close with a migration punch list**. Keep `MEASURED` (ctk-backed)
and `CONSIDER` (judgement-only) findings visibly separate — the reader's trust in the measured set depends
on not mixing in opinion.

---

## Target / Scope

- **Repo / branch / base:** `<repo>` · `<branch>` vs `<base>` (state the exact base ref measured).
- **What is public here:** published library / internal service / app — the call from Phase 0 that frames
  every severity (a `REMOVED` in a published crate ≠ one in an app).
- **Stack measured:** languages `ctk` read a contract from; note any changed files it could **not**
  (`unmeasured`) that need manual review.
- **Diff size:** files changed / insertions / deletions.

## Semver verdict

One line up top: **major** / **minor** / **none**, and why, in terms of the candidates. Start from
`ctk assess`'s `semver_impact`; if you downgraded it, say which candidate you discounted and why
(e.g. "the only `REMOVED` is a test-only symbol").

## Breaking findings (ranked, worst first)

Source these from `ctk assess` candidates where `breaking=true`. For each:

### <n>. <symbol> — <reason code>

- **Measurement:** the candidate. e.g. `process` `SIGNATURE_CHANGED` at `api.rs:40`;
  `before: pub fn process(req: Req) -> Resp` → `after: pub fn process(req: Req, opts: Opts) -> Resp`;
  `semver=major`. The before/after signature is the spine of the finding.
- **Breaks a real caller here?** 2-3 sentences of judgement — does this actually break downstream
  (required param vs defaulted, positional vs keyword rename, widened vs narrowed type, reachable vs
  test-only)? Confirm or downgrade.
- **Migration:** what a caller must change, or the deprecation/overload path that avoids a hard break.
  Point at the repo's existing convention if it has one.
- **Break class:** from `break-catalog.md` (the reason code maps to it).

## Additive surface (`ADDED`)

List the new public symbols (`ctk` `ADDED` candidates) with their signatures. These are the non-breaking
half — they justify a **minor** bump. Keep brief; do not treat as problems.

## Consider (judgement-only, no `ctk` candidate)

Real contract changes the tool does not measure (Phase 0/2 scope limits): public field/variant/constant
changes, TS/JS class methods without `export`, trait-method changes, and the exported surface of any
`unmeasured`-language file. Clearly separated so the reader knows these are manual judgement, not a `ctk`
measurement. Cite `file:line`.

## What was measured but held

Brief: public symbols that changed only cosmetically (reformatting, body, a downgraded positional rename)
and stayed compatible, and any files `ctk` could not read (so the reader knows coverage). "No breaking
changes" is a statement about _what was measured_, not a guarantee over the unmeasured set.

## Severity rubric (scope-relative)

- **Major** — any confirmed `REMOVED` / `VISIBILITY_REDUCED` / source-breaking `SIGNATURE_CHANGED` on a
  real contract symbol.
- **Minor** — only `ADDED` public surface; nothing breaks.
- **None** — no public contract change (only internal/private edits, reformatting, body churn).
- **Consider** — a change outside the tool's measured scope; manual judgement.

## Migration punch list (clear, actionable)

The closing section — for engineers, in priority order. No new findings; synthesize the above.

- **Lead with the bump.** One line: major / minor / none, and the single reason that forces it.
- **List what breaks callers**, each pointing at its finding and the concrete migration (add the overload,
  restore the export, keep the old param name).
- **List what's merely additive** (the minor-bump surface) and **what's unmeasured** and needs a manual
  contract look.
- **Stay honest.** If nothing breaks, say "no breaking changes, no major bump" and stop. Don't manufacture
  a break, and don't present unmeasured files as clean.
