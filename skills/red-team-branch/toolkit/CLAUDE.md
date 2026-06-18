# rtk — engineering practice

`rtk` is an offensive toolkit (Rust CLI) driven by the `red-team-branch` skill in
Phase 3 to break a branch against a **local** instance. It is a set of precise
probes, not a scanner. The whole tree is ~6k LOC and stays that way on purpose.

The non-negotiable rules below are enforced by shell gates, not opinion. Run
`make quality` before every commit and `make test` before you call work done.
A change that fails a gate is not "almost there" — it is broken.

## Principles

The tree stays small because of how we decide what to write, not just how we
split it.

- **YAGNI.** Build only what the probe in front of you needs. No speculative
  config keys, no "might want it later" parameters, no extension point without a
  second real caller. The skill drives a fixed contract — code to that, not to an
  imagined future.
- **KISS.** Prefer the plainest thing that works and reads top to bottom. A flat
  function beats a clever trait; an `if` beats a framework. Add an abstraction
  only when a second concrete use forces it, never before.
- **Standard library over custom code.** Reach for `std` before you write a
  helper, and long before you add a crate. If a method already does it —
  `trim_ascii_start`, `clamp`, `div_ceil`, `saturating_*`, `split_once`,
  `windows`, `is_loopback` — call it; don't reimplement it. Hand-roll only when
  std has no answer (PRNG, percent-encoding, statistics) or its answer is wrong
  for us (`DefaultHasher` isn't portable, so coverage fingerprints use a fixed
  FNV). When you do hand-roll, the comment says *why std won't do*.
- **Subtract before you add.** When code is bad, the strongest move is to delete
  it and replace the tangle with one correct line — quietly, no ceremony. Don't
  hide a bad approach behind a flag, comment it out "just in case," or stack a
  layer on top to paper over it. A negative diff that keeps the tests green *is*
  the improvement. Ship it.

## The driving contract (why most of the design exists)

An agent parses this binary. Hold the contract or the skill can't fold a result
into a finding:

- **stdout = exactly one JSON document.** Emit it through the single sink
  `cmd::emit(&json!({...}))`. Nothing else may touch stdout.
- **stderr = human progress and warnings** (`eprintln!`, `[rtk][WARN] ...`).
- **Local-only by default.** Every outbound request resolves through
  `cmd::base_spec` → `http::guard_target`, which refuses non-local hosts unless
  the host is in `safety.allow_hosts` or `--allow-remote` is passed. Never route
  a request around the guard.
- **No redirects followed** (a 3xx is an auth/IDOR tell worth seeing).
- **Deterministic.** No wall-clock or ambient randomness in logic. The fuzzer's
  PRNG is hand-rolled and `--seed`-replayable; findings carry a repro. Keep
  flaky signals (latency, transport) out of any novelty/coverage key.
- **Binary-free JSON.** Raw response bytes may be captured for oracles, never
  serialized.
- **Zero-setup.** Every config value is overridable by a flag, so an agent can
  drive the tool with no file. Don't add a knob that only works via the TOML.

## Architecture — small files, no god modules

- `src/main.rs` is a thin shell: parse CLI → load `Config` → build `Ctx` → apply
  flag overrides → dispatch. No business logic lives here.
- **Every subcommand is a module *directory*** under `src/cmd/`, never a single
  `.rs` file. Shape:
  - `mod.rs` — the `run(args, &ctx)` orchestrator + tiny local helpers, and
    `pub use args::Args;` to expose the surface.
  - `args.rs` — the `#[derive(clap::Args)]` struct, nothing else.
  - one file per responsibility (`request.rs`, `report.rs`, `probe.rs`,
    `classify.rs`, `crypto.rs`, ...). Split by what the code *does*.
  - `tests.rs` — inline unit tests (`#[cfg(test)] mod tests;`).
- **Shared concerns are directories too:** `config/` (load · model · ctx ·
  defaults · overrides), `http/` (client · request · response · send · guard ·
  url · status), `util/` (parse · text · stats · hash). Never collapse one back
  into a flat `src/config.rs`.
- Build requests with the `RequestSpec` builder
  (`RequestSpec::new(method, url).with_text_headers(..).with_body(..)`) and its
  `render(token, value)` for `{FUZZ}`/`{id}`/`{VAR}` substitution. Reuse it;
  don't hand-assemble requests in a command.

## Enforced structural gates (`make quality`)

These run on `src` and fail the build. Internalize them — don't discover them by
failing CI.

- **`make loc` — ≤ 100 lines per `.rs` file** (`src` *and* `tests`). If a file
  is growing, that's the signal to split it, not to shrink whitespace. The
  largest file today is 97 lines; treat 100 as a hard wall.
- **`make arch` — no god files.** A command must be a directory with a `mod.rs`
  (no `src/cmd/foo.rs`); shared modules must be directories (no `src/http.rs`).
- **`make magic` — no magic literals in production.** Counts, bounds, ranges,
  retry/`take(N)` args, RNG bounds, and the high-churn clap defaults
  (HTTP method, param location, port-ish, method lists) must be **named
  constants** (see `config/defaults.rs`, `const MAX_RESULT_ROWS`).
- **`make positionals` — no positional contracts in production.** No tuple field
  access (`.0`/`.1`), no tuple type aliases, no multi-value tuple returns. Return
  a **named struct** instead (e.g. `SignedInput`, `Summary`) — the field names
  are the documentation.

`tests.rs` and the `tests/` tree are exempt from `magic`/`positionals` (tuples
and inline literals are fine in tests), but **not** from the 100-LOC ceiling.

## Testing — behaviour, not implementation

- Integration tests in `tests/` spin up a real `tiny_http` server on an
  ephemeral `127.0.0.1:0` port, run the **actual compiled binary**
  (`CARGO_BIN_EXE_rtk`) via the shared `common::{run_rtk, write_config,
  start_server}`, and assert on the parsed-JSON result. Test what the agent
  sees, not internal calls.
- `run_rtk` panics if stdout isn't valid JSON — that alone guards the contract.
- Crypto is pinned to **known-answer vectors** (the HMAC vector in `make smoke`
  and `tests/sign.rs`). Security-critical helpers like `guard_target` get
  **adversarial** unit tests (lookalike-bypass hosts: `127.0.0.1.evil.com`,
  `169.254.169.254`, `localhost.evil.com`).
- Don't write tests that restate the implementation — they give zero confidence.
- `make test` runs `quality` first, then `cargo test`. Order is intentional.

## Errors & dependencies

- `anyhow::Result` throughout. Refuse loudly with `anyhow::bail!` and an
  **actionable** message (e.g. "Add safety.allow_hosts or pass --allow-remote
  with authorization."). No silent failures, no swallowed errors, no fallback
  that hides a problem.
- Lean dependency set (clap, serde, tokio, reqwest+rustls, hmac/sha2, anyhow).
  The mutation fuzzer is deliberately **zero-external-crate** (hand-rolled PRNG
  and mutation ops). Don't reach for a crate to do what a small, auditable
  function can.
- No prebuilt binary is shipped — offensive tooling is built from source you can
  audit. `make setup` builds it; `make smoke` self-checks via the HMAC vector.

## Adding a subcommand

1. `src/cmd/<name>/` with `mod.rs` (`run`) + `args.rs` (`Args`) + responsibility
   files + `tests.rs`.
2. Wire `pub mod <name>;` into `src/cmd/mod.rs` and add the variant + dispatch
   arm in `src/main.rs`.
3. Emit one JSON doc via `cmd::emit`; route every request through
   `cmd::base_spec` so the local-only guard always applies.
4. Add an integration test that drives the real binary against `start_server`.
5. `make quality && make test` must pass.

## House style

- Doc comments (`//!` module, `///` item) state the *contract and the why*,
  tersely. Comments are ≤ 2 lines; no prose paragraphs.
- Enums for closed variants (`RequestBody`, `RequestHeader`); `match` with
  explicit arms; small pure functions over long ones.
- The user's global conventions (red-green TDD, concise bullet-point PRs, the
  attribution trailer) apply here too — this file does not restate them.
