# cqt — engineering practice

`cqt` is the deterministic measurement engine (Rust CLI) driven by the
`quality-branch` skill. It computes complexity metrics, branch-scoped deltas,
repo percentiles, churn × complexity hotspots, duplication, temporal coupling,
and the unified `assess` candidate list — and **nothing else**. It does not
judge code; it produces numbers the skill ranks and cites. The tree stays small
on purpose.

The non-negotiable rules below are enforced by shell gates, not opinion. Run
`make quality` before every commit and `make test` before you call work done. A
change that fails a gate is not "almost there" — it is broken.

## Principles

The tree stays small because of how we decide what to write, not just how we
split it.

- **YAGNI.** Build only what the signal in front of you needs. The skill drives
  a fixed contract — code to that, not to an imagined metric nobody asked for.
  We deliberately ship **no TOML config**: every value is a flag or a named
  default. Don't add a config layer back.
- **KISS.** The plainest thing that reads top to bottom. A flat function beats a
  clever trait; an `if` beats a framework. Add an abstraction only when a second
  concrete use forces it — that is exactly why `engine/pair.rs` and
  `cmd::functions_at` exist (delta *and* assess both need them), and why nothing
  speculative does.
- **Standard library over custom code, but reuse the engine.** The one heavy
  dependency is the metrics engine (`rust-code-analysis`, pinned by SHA);
  everything else — git access, percentiles, JSON shaping, the duplication
  lexer/fingerprinter, the coupling tally — is `std` + serde. The fuzz-free FNV
  hash and the tokenizer are hand-rolled because std has no answer and a small
  auditable function beats a crate.
- **Subtract before you add.** A negative diff that keeps tests green is the
  improvement. When the pairing logic was duplicated, it moved to one shared
  `engine/pair.rs` and the old copy was deleted — not flagged, not commented
  out. Ship the deletion.

## The driving contract (why most of the design exists)

An agent parses this binary. Hold the contract or the skill can't fold a result
into a finding:

- **stdout = exactly one JSON document.** Emit it through the single sink
  `cmd::emit(&json!({...}))`. Nothing else may touch stdout — no `println!`,
  no `dbg!`.
- **stderr = human progress and warnings** (`eprintln!`).
- **Deterministic.** Every signal is a pure function of repo state. No
  wall-clock, no dates, no randomness, no sampling. The parallel scan
  (`engine/scan.rs`) collects then **sorts by path**, so thread scheduling never
  changes output. Churn and coupling are commit *counts*, never time windows.
  Any `HashMap` is sorted (or its results are) before it reaches stdout — never
  iterate a map straight into the output.
- **Schema-versioned.** Multi-field reports (`assess`, `dup`, `coupling`) carry
  a `schema` string. Bump it when the shape changes.
- **Parse honesty.** Unknown language, a failed parse, or an oversized/binary
  file yields `parse_ok=false` / a skip / `parse_errors` — never a silent zero.
  Callers downgrade on it; never fabricate a metric.
- **Zero-setup.** Every value is overridable by a flag, so an agent drives the
  tool with no file. Don't add a knob that only works via a (non-existent) file.

## Architecture — small files, no god modules

- `src/main.rs` is a thin shell: parse CLI → resolve repo+base into `Ctx` →
  dispatch. No business logic.
- **Every subcommand is a module directory** under `src/cmd/`: `mod.rs` (the
  `run(args, &ctx)` orchestrator), `args.rs` (the `clap::Args` struct only), and
  one file per responsibility (`pair.rs`, `score.rs`, `reason.rs`, `build.rs`,
  `detect.rs`, `compute.rs`, `log.rs`, `model.rs`). Split by what the code *does*.
- **Shared concerns are directories too:** `engine/` (the rust-code-analysis
  binding — `analyze` · `flatten` · `model` · `pair` · `scan`), `git/` (`repo` ·
  `diff` · `blob` · `churn` + the `git()` runner), `util/` (`stats` ·
  `defaults`). Never collapse one into a flat `src/engine.rs`.
- The engine binding is isolated in `engine/`: `analyze(path, bytes)` is the one
  entry point, and bytes may come from the working tree or a git blob, so the
  same code measures both sides of a delta with no temp files. `engine/scan.rs`
  owns the single guarded source reader (size + binary skip) and the
  deterministic parallel map.

## Enforced structural gates (`make quality`)

Internalize these — don't discover them by failing CI.

- **`make loc` — ≤ 100 lines per `.rs` file** (`src` *and* `tests`). A growing
  file is the signal to split, not to shrink whitespace. The largest today is
  `cmd/dup/detect.rs` at 99 — treat 100 as a hard wall.
- **`make arch` — no god files.** A command is a directory with `mod.rs`; shared
  modules (`engine`, `git`, `util`, `config`) must be directories, never flat.
- **`make magic` — no magic literals in production.** clap defaults,
  `take`/`skip`/`truncate` caps, and numeric ranges must be named constants. Put
  domain constants next to their use (`DUP_K` in dup, `MAX_OCC` in detect,
  parallel bounds in scan, coupling thresholds in coupling/args) — locality over
  one god `defaults.rs`; only the truly shared ones (percentiles, EPS) live in
  `util/defaults.rs`.
- **`make positionals` — no positional contracts in production.** No tuple field
  access (`.0`/`.1`), no tuple type aliases, no multi-value tuple returns.
  Return a named struct — the field names are the documentation (`Run{a0,a1,..}`,
  `PairKey{a,b}`, `FnPair`, `Candidate`). (The gate's leading char-class excludes
  float literals like `0.0`, which are not tuple access.)

`tests/` is exempt from magic/positionals (literals/tuples are fine in tests) but
**not** from the 100-LOC ceiling.

## Testing — behaviour, not implementation

- Integration tests in `tests/` build a throwaway git repo (via the `tempfile`
  dev-dependency and real `git` with pinned commit dates), run the **actual
  compiled binary** (`CARGO_BIN_EXE_cqt`), and assert on the parsed JSON. Test
  what the agent sees, not internal calls.
- `run_cqt` panics if stdout isn't valid JSON — that alone guards the contract.
- Every command has a behavioural test that proves its *claim*: delta proves
  nesting raises cognitive and a rename isn't split into add+remove; dup proves a
  Type-2 clone is found and `--type1` correctly does not; coupling proves the
  missing co-change is flagged; determinism proves two runs are byte-identical.
- Don't write tests that restate the implementation — a delta test asserts the
  branch's increase is reported, not that a specific magic number comes back.
- `make test` runs `quality` first, then `cargo test`. Order is intentional.

## Errors & dependencies

- `anyhow::Result` throughout; refuse loudly with an **actionable** message
  (e.g. "not a git repo; pass --repo"). No silent failures, no fallback that
  hides a problem. Per-file IO failures inside a scan are a *skip* (with the
  honesty flag), not a crash.
- Lean deps: clap, serde, anyhow, tempfile (dev), and the pinned
  `rust-code-analysis` engine. Pin it by **commit SHA**, never a moving branch —
  reproducible builds are part of the determinism story. Git access is
  `std::process::Command` with `-c core.quotepath=false`, no git crate. No
  prebuilt binary is shipped; `make setup` builds it.

## Adding a subcommand

1. `src/cmd/<name>/` with `mod.rs` (`run`) + `args.rs` (`Args`) + responsibility
   files + `model.rs`.
2. Wire `pub mod <name>;` into `src/cmd/mod.rs` and add the variant + dispatch
   arm in `src/main.rs`.
3. Emit one JSON doc via `cmd::emit`; keep every signal deterministic and carry a
   `schema` string if the shape is non-trivial.
4. Add an integration test that drives the real binary against a git fixture and
   proves the command's claim.
5. `make quality && make test` must pass.

## House style

- Doc comments (`//!` module, `///` item) state the *contract and the why*,
  tersely. Comments ≤ 2 lines; no prose paragraphs.
- Enums for closed variants (`Change`, `CommentStyle`); `match` with explicit
  arms; small pure functions over long ones.
- The user's global conventions (red-green TDD, concise bullet-point PRs, the
  attribution trailer) apply here too — this file does not restate them.
