# Code-Quality Smell Catalog

A vocabulary of maintainability smell classes, each mapped to the **deterministic `cqt` signal that
surfaces a candidate** and the **judgement** needed to confirm it. The signal makes the candidate
reproducible; you decide whether it is a real problem in this repo. Cite the signal in every finding.

The order is roughly by how strongly the signal supports a finding.

---

## 1. Complexity regression (branch made a function harder to understand)

- **Signal:** `cqt delta` — a function with a large `delta_cognitive` / `delta_cyclomatic`, especially one
  whose `after_cognitive` crosses the repo p95–p99 from `cqt calibrate`.
- **Judgement:** is the control flow genuinely tangled now (deep nesting, many branches), or did it grow
  for a legitimate reason (a real new case)? Cognitive complexity weights nesting, so a high score usually
  means "hard to follow", not just "big".
- **Remedy direction:** extract a helper or guard clause, flatten nesting, replace a condition chain with a
  typed model/dispatch. Point at how other functions in this repo stay flat.

## 2. Hotspot aggravation (fragile code getting more fragile)

- **Signal:** `cqt hotspot` — a file with high churn × complexity and `in_branch=true`, i.e. the branch is
  editing code that is both complex and changed often (where defects concentrate).
- **Judgement:** does the change add to the tangle, or is it a step toward untangling it? A complexity
  *increase* on a top hotspot is the high-risk case.
- **Remedy direction:** prioritize decomposing this file; a small refactor here buys more than elsewhere.

## 3. New complexity introduced at birth

- **Signal:** `cqt delta` `status=added` with `after_cognitive` already at a high repo percentile.
- **Judgement:** is the new function born tangled, or is the domain genuinely complex? Compare to peers via
  `calibrate`.
- **Remedy direction:** split responsibilities before merge; a new function should rarely enter above p90.

## 4. God function / oversized unit

- **Signal:** `cqt metrics` — large `sloc` combined with high cognitive on a single function; or a file
  whose function count and summed complexity dwarf its peers.
- **Judgement:** is it doing several jobs? Many maintainability bars (and the repo's own, from Phase 0)
  treat past a size/complexity point as a smell.
- **Remedy direction:** extract cohesive sub-functions or modules; keep one reason-to-change per unit.

## 5. Repeated conditional / missing model

- **Signal:** `cqt delta` shows several functions in the diff each gaining branches (cyclomatic up across
  siblings) for the same distinction.
- **Judgement:** the same `if kind == ...` scattered across functions usually signals a missing type/enum
  or dispatch. The tool shows the spread; you spot the shared shape.
- **Remedy direction:** introduce the model/dispatch once; the per-site branches collapse.

## 6. Thin wrapper / needless indirection

- **Signal (weak/indirect):** `cqt delta` `status=added` functions with cognitive ~0 and tiny `sloc` that
  only forward to another call.
- **Judgement:** mostly judgement — does the wrapper clarify an API or just add a hop? The metric only
  flags "trivially small new function"; you decide if it earns its keep.
- **Remedy direction:** inline it unless it names a real boundary.

## 7. Large argument list / many exit points (structural biomarkers)

- **Signal:** `cqt assess` reason `LARGE_ARG_LIST` (params ≥ threshold) or `MANY_EXIT_POINTS` (exits ≥
  threshold); raw `params`/`exits` in `cqt metrics`.
- **Judgement:** these catch what a single complexity number misses — a 7-parameter function can be
  cognitively "simple" yet painful to call and test. Is the parameter list a missing parameter-object, or
  the exits a tangle of early returns?
- **Remedy direction:** introduce a parameter struct / builder; consolidate exits behind a single result.
- **Note:** these thresholds are absolute by design (a wide signature is hard regardless of repo),
  distinct from the repo-relative complexity ranking.

## 8. Branch-introduced duplication (copy-paste)

- **Signal:** `cqt dup` (default branch scope) — a clone group where a branch-changed file participates.
  Copy-paste-and-modify against existing repo code (added ⇄ existing) is the highest-defect form.
- **Judgement:** is this the bad kind of duplication (shared logic that will now drift), or incidental
  structural similarity? Keywords/operators are kept verbatim so structure differentiates, but you decide
  if extraction is warranted.
- **Remedy direction:** extract the shared block into a function/module; if it is genuinely independent,
  leave it and say why.

## 9. Missing co-change (likely-forgotten edit)

- **Signal:** `cqt coupling` (default branch scope) — a pair with `status: "missed"`: the branch changed
  one file but not its historical co-change partner, which usually changes with it.
- **Judgement:** did the author forget the partner, or did they intentionally break a coupling? High
  `degree` (e.g. ≥ 0.8) on a `missed` pair is the strong signal. This is an architectural/correctness hint,
  not a complexity smell.
- **Remedy direction:** check whether the partner needs the same change, or whether the invariant linking
  them still holds.

## 10. Parse-unreliable change (coverage gap, not a smell)

- **Signal:** `parse_ok=false` / `parse_errors>0` / skipped for a changed file.
- **Judgement:** none — this is an honesty flag. Do **not** raise a finding on this file; report it under
  coverage so the reader knows it was not measured (unsupported language, lossy parse, oversized, binary).

---

## Using the catalog

- A finding names exactly one primary smell class and cites the `cqt` signal that surfaced it. `cqt
  assess` reason codes map directly: `REGRESSION_ABOVE_P95`/`COMPLEXITY_REGRESSION`→1,
  `COMPLEXITY_IN_HOTSPOT`/`NEW_IN_HOTSPOT`→2, `NEW_COMPLEXITY_HIGH`/`NEW_FUNCTION`→3,
  `LARGE_ARG_LIST`/`MANY_EXIT_POINTS`→7, `IMPROVED`→a credited cleanup.
- Classes 1-3, 8, 9 are strongly measurement-backed → `MEASURED` findings. Classes 4-5, 7 are
  measurement-assisted. Class 6 is mostly judgement → usually a `CONSIDER`. Class 10 is coverage, never a
  finding.
- The repo's own linters (Phase 0) already cover many micro-smells (naming, formatting, unused) — defer to
  them and do not duplicate. `cqt` adds the cross-language, diff-scoped, repo-calibrated complexity,
  duplication, and coupling layers.
