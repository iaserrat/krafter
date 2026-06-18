# Changelog

All notable changes to krafter are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to
adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
