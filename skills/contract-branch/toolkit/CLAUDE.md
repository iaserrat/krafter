# ctk — engineering practice

`ctk` is the deterministic measurement engine (Rust CLI) driven by the
`contract-branch` skill. It extracts a file's **public contract surface** (every
public function, method, type, field, variant, constant, and alias, each with a
normalized signature and a visibility), diffs that surface across two git refs,
and emits **reason-coded breaking-change candidates** with an overall semver
impact and an optional CI gate — and **nothing else**. It does not decide what is
"truly breaking"; it produces the facts the skill ranks and cites. The tree stays
small on purpose.

The non-negotiable rules below are enforced by shell gates, not opinion. Run
`make quality` before every commit and `make test` before you call work done. A
change that fails a gate is not "almost there" — it is broken.

## Principles

- **Agnostic by construction.** Extraction is **one uniform mechanism**
  (tree-sitter parses every language) plus **per-language rules as data** — small
  walks over real parse-tree node kinds in `surface/rules/`. We do not hand-roll
  parsers per language; if a language needs support, add its grammar + a rules
  module, never a brace-counting scanner.
- **YAGNI.** Build only what the contract diff needs. We ship **no TOML config**:
  every value is a flag or a named default. Don't add a config layer.
- **KISS.** Each language's rules are a flat walk that matches node kinds and
  emits symbols. The visibility rule is a single predicate per language
  (`pub`/`export`/`public`/capitalization/underscore). Add an abstraction only
  when a second concrete use forces it.
- **Subtract before you add.** A negative diff that keeps tests green is the
  improvement. ctk was forked from cqt and then *shed* everything it did not
  need — the metrics engine, percentile stats, the duplication lexer, churn —
  and later replaced its hand-rolled signature scanner and per-language
  visibility hacks with the tree-sitter rules. Ship the deletion.

## The driving contract (why most of the design exists)

An agent parses this binary. Hold the contract or the skill can't fold a result
into a finding:

- **stdout = exactly one JSON document.** Emit it through the single sink
  `cmd::emit(&json!({...}))`. Nothing else may touch stdout — no `println!`,
  no `dbg!`.
- **stderr = human progress and warnings** (`eprintln!`).
- **Deterministic.** Every signal is a pure function of repo state. No
  wall-clock, no randomness. tree-sitter parses deterministically; the symbol
  list is sorted (line, name); candidates are sorted (breaking first, then reason
  rank, path, line, symbol). The base side is read from a git blob, the head side
  from the working tree.
- **Formatter-proof.** Signatures are compared on a whitespace-insensitive key
  (`sig::canonical`) that also eats a wrapped list's trailing comma, so
  reformatting, reordering members, and rewriting bodies never read as a contract
  change. Members match by qualified name, so reordering fields/variants is silent.
- **Schema-versioned.** Reports carry a `schema` string (`ctk.assess/v2`,
  `ctk.surface/v1`). Bump it when the shape changes.
- **Parse / contract honesty.** An unknown extension or a grammar that fails to
  load yields `vis_supported=false` (reported `unmeasured`); a syntax error
  yields `parse_ok=false` with whatever parsed. Never a silent empty contract.
- **Zero-setup.** Every value is a flag; an agent drives the tool with no file.

## Architecture — small files, no god modules

- `src/main.rs` is a thin shell: parse CLI → resolve repo+base into `Ctx` →
  dispatch. No business logic.
- **Every subcommand is a module directory** under `src/cmd/`: `mod.rs` (`run`),
  `args.rs` (the `clap::Args` only), and one file per responsibility (`assess`
  splits into `classify`, `load`, `gate`, `model`).
- **`surface/` is the extraction engine:** `lang.rs` (extension → language +
  grammar), `parse.rs` (tree-sitter parse), `node.rs` (node text / field / header
  helpers), `sig.rs` (normalize + the formatter-proof compare key), and
  `rules/` — one walk per language (`rust`, `web`, `python`, `go`, each with a
  `_member` companion for type members) plus `common.rs` (the `Symbol` builder).
  `surface::extract` is the one entry point and works on bytes from a blob or the
  working tree.
- **Shared infra stays directories too:** `git/` (`repo` · `diff` · `blob` + the
  `git()` runner — `diff` keeps deletes, unlike cqt), `util/` (`defaults` ·
  `source`, the size/binary-guarded reader). Never collapse one into a flat file.

## Enforced structural gates (`make quality`)

- **`make loc` — ≤ 100 lines per `.rs` file** (`src` *and* `tests`). A growing
  file is the signal to split (that is why each language has a `_member`
  companion), not to shrink whitespace. Treat 100 as a hard wall.
- **`make arch` — no god files.** A command is a directory with `mod.rs`; shared
  modules (`engine`, `git`, `util`, `config`) must be directories.
- **`make magic` — no magic literals in production.** clap defaults,
  `take`/`skip`/`truncate` caps, and numeric ranges must be named constants
  (e.g. `EXIT_GATE` in `assess/gate.rs`, the size guards in `util/source.rs`).
- **`make positionals` — no positional contracts in production.** No tuple field
  access (`.0`/`.1`), no tuple type aliases, no multi-value tuple returns. Return
  a named struct (`Symbol`, `Candidate`, `ChangedFile`, `Loaded`, `Run`).

`tests/` is exempt from magic/positionals but **not** the 100-LOC ceiling.

## Testing — behaviour, not implementation

- Integration tests in `tests/` build a throwaway git repo (via `tempfile` and
  real `git`), run the **actual compiled binary** (`CARGO_BIN_EXE_ctk`), and
  assert on the parsed JSON. Test what the agent sees.
- Every claim has a test: each reason code fires (`REMOVED`, `SIGNATURE_CHANGED`,
  `VISIBILITY_REDUCED`, `REMOVED_DEPRECATED`, `ADDED`); type members (fields,
  variants, consts) diff; all four languages extract; the `--fail-on` gate and
  `--baseline` suppression behave; reformatting/reordering/body churn is silent;
  two runs are byte-identical.
- Pure helpers (`sig::normalize`/`canonical`, `deprecate`) carry inline
  `#[cfg(test)]` unit tests for their edge cases.
- `make test` runs `quality` first, then `cargo test`. Order is intentional.

## Errors & dependencies

- `anyhow::Result` throughout; refuse loudly with an **actionable** message. A
  missing/binary/oversized blob is an empty surface (with the honesty flag), not
  a crash. A missing `--baseline` file is a hard error — silently ignoring it
  would let a stale baseline hide breaks.
- Lean deps: clap, serde, anyhow, tempfile (dev), and tree-sitter + the grammar
  crates (`tree-sitter-rust`/`-typescript`/`-python`/`-go`), pinned to a set
  compatible with tree-sitter 0.24. Git access is `std::process::Command`, no git
  crate. No prebuilt binary is shipped; `make setup` builds it.

## Adding a language

1. Add its `tree-sitter-<lang>` grammar to `Cargo.toml` and a `Lang` variant +
   extension mapping + `grammar()` arm in `surface/lang.rs`.
2. Add `surface/rules/<lang>.rs` (and `<lang>_member.rs` if it has type members):
   a walk that matches node kinds and emits `Symbol`s via `common::symbol`, with
   the language's visibility predicate.
3. Wire the dispatch arm in `surface/rules/mod.rs`.
4. Add an integration test proving a break is caught and a private change is not.
5. `make quality && make test` must pass.

## Contract scope (honest limits — keep them in the report)

- **Languages:** Rust, TypeScript/JavaScript (TSX grammar covers JS/JSX), Python,
  Go. Others are reported `unmeasured`.
- **Signature-level, not type-resolved.** A changed parameter *type* shows as
  `SIGNATURE_CHANGED`; the tool flags the candidate and the skill judges whether
  it is source-breaking. A parameter *rename* also flags — breaking for
  keyword-arg callers (Python), cosmetic for positional ones. Codes, not verdicts.
- **Known gaps to state when relevant:** Rust trait-impl methods (the trait
  defines the contract, not the impl); a `pub` member of a private type; Go
  multi-name const/var specs (first name only); TS re-exports (`export { x }`).

## House style

- Doc comments (`//!` module, `///` item) state the *contract and the why*,
  tersely. Comments ≤ 2 lines.
- Enums for closed variants (`Reason`, `Visibility`, `Lang`); `match` with
  explicit arms; small pure functions over long ones.
- The user's global conventions (red-green TDD, concise PRs, the attribution
  trailer) apply here too — this file does not restate them.
