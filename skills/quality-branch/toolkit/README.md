# cqt — code-quality toolkit

A self-contained Rust CLI that produces the **deterministic measurements** the
`quality-branch` skill judges from. It computes per-function complexity,
branch-scoped deltas, repo-relative percentiles, churn × complexity hotspots,
duplication, temporal coupling, and a unified ranked candidate list. stdout is
exactly one JSON document (parse it); stderr is human progress.

It does not decide whether code is "good" — it measures, so the skill can rank
and the verdict stays reproducible.

## Build

```bash
make setup        # verifies cargo, builds target/release/cqt
make test         # structural gates (loc/arch/magic/positionals) then cargo test
```

The metrics engine is a pinned commit of Mozilla's `rust-code-analysis`
(tree-sitter based, multi-language: Rust, Python, JS/TS, C/C++, Go, Java, and
more). The first build compiles the grammars (needs a C compiler) and takes a
few minutes; afterwards it is cached.

## Global flags

| Flag | Meaning |
|---|---|
| `--repo <path>` | repo root (default: `git rev-parse --show-toplevel` of the cwd) |
| `--base <ref>` | base ref the branch is measured against (default: `origin/HEAD`, else `main`/`master`) |

## Commands

| Command | Shows | Needs git? |
|---|---|---|
| `cqt assess` | **the spine:** ranked, reason-coded candidate findings (delta × percentile × hotspot × biomarkers) | yes |
| `cqt delta` | functions added/changed/removed, before→after cognitive/cyclomatic, sorted by Δcognitive | yes |
| `cqt calibrate` | the repo's own cognitive/cyclomatic percentile distribution (p50–p99, max) | yes |
| `cqt hotspot [--window N] [--top N]` | files ranked by churn × complexity; `in_branch` flags branch-touched ones | yes |
| `cqt dup [--scope branch\|repo] [--type1]` | duplicated spans; default scope = clones the branch introduces | yes |
| `cqt coupling [--scope branch\|global]` | files that historically co-change; default surfaces the **missing co-change** | yes |
| `cqt metrics --paths <f>...` | per-function cyclomatic/cognitive/sloc/params/exits for given files | no |

Every result includes a parse-honesty signal (`parse_ok` / `parse_errors` /
skipped). A file whose language is unsupported, that parsed lossily, or that is
oversized/binary is reported as unmeasured, never as zero-complexity.

## Worked examples

The headline — one command joins every signal into ranked candidate findings:
```bash
cqt --base main assess
# -> { "command":"assess","schema":"cqt.assess/v1","base":"main","candidates":[
#   { "path":"pay.rs","function":"process","start_line":40,"end_line":92,
#     "direction":"regression","reasons":["COMPLEXITY_REGRESSION","REGRESSION_ABOVE_P95","COMPLEXITY_IN_HOTSPOT"],
#     "before_cognitive":12.0,"after_cognitive":41.0,"delta_cognitive":29.0,
#     "cognitive_percentile":98.7,"params":4.0,"exits":3.0,"churn":47,"in_hotspot":true,
#     "rank":{"score":...,"formula":"percentile * (1 + max(0, delta_cognitive)) * (1 + churn)",
#             "percentile_term":98.7,"delta_term":30.0,"churn_term":48.0} }, ... ] }
```
Each candidate is a self-contained evidence bundle: an anchor (`file:line`),
every raw measurement separately (never blended), repo-relative percentile, a
transparent rank (formula + per-term values), reason codes (not verdicts), and a
direction (`new`/`regression`/`improved`/`changed`) so cleanups get credit.

Repo distribution to rank against (no absolute thresholds):
```bash
cqt calibrate
# -> { "command":"calibrate","functions":812,
#      "cognitive":{"p90":9.0,"p95":15.0,"p99":28.0,"max":63.0}, "cyclomatic":{...} }
```

Duplication the branch introduces (cross-file, Type-1 + Type-2):
```bash
cqt --base main dup
# -> { "command":"dup","schema":"cqt.dup/v1","scope":"branch",
#      "params":{"k":12,"min_tokens":50,"min_lines":5,"type2":true},
#      "clone_groups":[ { "token_length":184,"line_length":41,
#        "members":[{"path":"a.rs","start_line":12,"end_line":53},{"path":"b.rs","start_line":88,"end_line":129}] } ] }
```

Missing co-change ("you changed A but not B, which usually rides along"):
```bash
cqt --base main coupling
# -> { "command":"coupling","schema":"cqt.coupling/v1","scope":"branch",
#      "pairs":[ { "file_a":"auth/session.rs","file_b":"auth/token.rs",
#        "shared_commits":28,"revs_a":31,"revs_b":34,"degree":0.9032,
#        "status":"missed","anchor":"auth/session.rs" } ] }
```

## Determinism contract

- **stdout = one JSON document** (via a single emit sink); **stderr = progress.**
- **Deterministic:** every signal is a function of repo state. No wall-clock, no
  dates, no sampling — same repo + same base → byte-identical output. The
  repo-wide scan is parallel but sorts by path; churn/coupling use commit counts.
- **Local source only:** reads the working tree and git blobs; makes no network
  calls. Oversized (>1 MiB) and binary files are skipped deterministically.

`cqt --help` and `cqt <command> --help` are authoritative.

## Scope notes (honest limits)

- `dup` v1 detects **cross-file** clones; within-file duplication is future work.
  Common k-grams above an occurrence cap are skipped to bound cost.
- `coupling` is computed over full history; the mega-commit guard
  (`--max-changeset`) drops sweeping commits that would fabricate coupling, so a
  repo whose only history is a squashed import reports `commits_analyzed: 0`.
- Defaults (`dup` 50 tokens, `coupling` 5 shared commits / 0.30 degree) suit
  mid-size repos; scale thresholds up on large monorepos.
