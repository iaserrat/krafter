# Changelog

All notable changes to krafter are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to
adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-06-19

### Added

- **Browser-ready HTML report view in all three skills.** After writing the
  Markdown report, each skill now renders a single self-contained HTML file in a
  minimal editorial style (**Tailwind** via the Play CDN), saves it to a stable
  `/tmp/<skill>-<branch>.html` path — overwritten on rerun so reruns refresh
  rather than accumulate — and opens it in the browser (`open` / `xdg-open`). The
  Markdown report in the repo root remains the source of record.

### Changed

- **Brought all three skills to structural parity on toolkit provisioning.** The
  toolkit build now lives in **Phase 0 (Recon)** in every skill, with the identical
  idempotent step `test -x toolkit/target/release/<bin> || (cd toolkit && make setup)`.
  Previously `red-team-branch` deferred the `rtk` build to Phase 3 (finding-gated)
  and offered a "build-free path" in Phase 0a as an equal option — so on a branch
  assessed clean in-branch the agent would skip the toolkit and fall back to
  hand-rolled `curl`. The build is now hoisted to Phase 0a and the manual
  `docker ps`/`lsof` recon is demoted to a true fallback (only when Rust is
  unavailable).
- **`red-team-branch`: credential-free baseline now runs on every assessment**,
  regardless of findings. `rtk headers`, `cors`, `discover`, and `params` are
  framed as recon hygiene that fires even on a "clean" branch, not as proofs gated
  on a confirmed finding.
- **`red-team-branch`: the dynamic-proof ledger gate now checks the proof
  mechanism.** Each PROVEN/held row records the exact `rtk` subcommand used; a
  hand-rolled `curl` proof for a class that has an `rtk` subcommand does not pass
  the gate. `curl`/breakpoint is legitimate only for an uncovered class, a
  destructive sink, or a missing toolchain — and the row must name which.

## [0.1.2] - 2026-06-18

### Added

- **`contract-branch` skill** with the bundled `ctk` Rust tool: deterministic,
  formatter-proof breaking-change review of a branch's diff. Built on
  **tree-sitter** — one uniform parse mechanism with per-language contract rules
  as data. Extracts the full public contract surface (functions, methods, types,
  struct/Go fields, enum variants, constants, and aliases, with normalized
  signatures and visibility) at the base and the branch head across **Rust,
  TypeScript/JavaScript, Python, and Go**, diffs them, and emits reason-coded
  candidates — `REMOVED`, `REMOVED_DEPRECATED`, `SIGNATURE_CHANGED`,
  `VISIBILITY_REDUCED`, `ADDED` — each proved by a `before → after` signature at
  `file:line`, with an overall semver impact. Removal of an already-deprecated
  symbol is surfaced distinctly and ranked lower.
- **CI gate for `ctk assess`:** report-only by default; `--fail-on major|minor|any`
  exits non-zero as a merge gate, and `--baseline <file>` excludes accepted breaks
  from the gate while keeping them in the report.
- Reformatting, reordering members, and body rewrites never flag — only real
  token changes surface; unsupported languages and syntax errors are reported
  honestly rather than as a clean contract.

### Changed

- **`red-team-branch`: dynamic proof is now enforced, not optional.** Phase 3
  read-only proofs (`sweep --compare`, `matrix` GETs, `headers`, `cors`,
  `gql --introspection`, `timing`, `discover`) run automatically against a
  confirmed-local instance with no confirmation prompt; only mutating or
  destructive proofs (and any non-local target) still confirm first. A mandatory
  pre-verdict **dynamic proof ledger** blocks the verdict while any CONFIRMED
  critical/high finding or kill chain is still `NEEDS-DYNAMIC` without a named
  blocker, so the skill proves exploitation by default instead of on request. The
  report template gains the ledger section.

## [0.1.1] - 2026-06-18

### Added

- **Self-hosted marketplace** (`.claude-plugin/marketplace.json`): krafter now
  serves as its own marketplace, so it installs through the standard flow
  (`/plugin marketplace add iaserrat/krafter` → `/plugin install krafter@krafter`)
  and updates via `/plugin update krafter` — no manual workarounds needed.
- README install instructions and updated layout diagram.

## [0.1.0] - 2026-06-18

First public release.

### Added

- **`quality-branch` skill** with the bundled `cqt` Rust tool: deterministic
  code-quality review of a branch's diff. Measures per-function complexity
  deltas, repo-relative percentiles, churn × complexity hotspots,
  branch-introduced duplication, and dropped co-change coupling, then ranks the
  findings. Every finding cites a measurement at `file:line`.
- **`red-team-branch` skill** with the bundled `rtk` Rust tool: offensive,
  exploit-first security review of a branch's diff. Models the attacker's
  black-box view, confirms exploitability against the source, chains findings
  into kill chains, and proves them against a local instance. Local-only by
  default.
- Plugin manifest, README, and MIT license.
