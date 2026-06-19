```
    ██   ██ ██████   █████  ███████ ████████ ███████ ██████
    ██  ██  ██   ██ ██   ██ ██         ██    ██      ██   ██
    █████   ██████  ███████ █████      ██    █████   ██████
    ██  ██  ██   ██ ██   ██ ██         ██    ██      ██   ██
    ██   ██ ██   ██ ██   ██ ██         ██    ███████ ██   ██

             no measurement, no finding
```

# krafter

**A growing family of evidence-driven reviewers for the diff on your branch — each
one grounds every finding in a real measurement at a real `file:line`, never an
opinion.**

krafter is a [Claude Code](https://claude.com/claude-code) plugin. Most review
tooling hands you opinions; krafter hands you measurements. Each skill runs a
bundled Rust tool against your branch and grounds every finding in a real number
at a real `file:line`. If it can't measure it, it doesn't report it.

## Philosophy

- **Measure before you judge.** Subjective calls are fine once the evidence under
  them is deterministic and reproducible across runs.
- **Every finding cites a `file:line`.** No measurement, no finding.
- **It reports, you decide.** krafter assesses and proves; it never edits or
  patches your code.

## Skills

### 🔬 quality-branch — did this branch make the code worse?

A bundled Rust tool (`cqt`) measures the diff and ranks the findings:

- Per-function complexity deltas, scored against the rest of the repo
- Churn × complexity hotspots
- Branch-introduced duplication
- Co-change coupling that got dropped

**Use it for:** PR quality gates, "what got more complex," "where's the risky
code," reproducible maintainability assessments.
**Not for:** style nits, formatting, or applying the fixes. It assesses, never
fixes.

### 🗡️ red-team-branch — is this branch exploitable?

A bundled Rust tool (`rtk`) treats the diff as hostile and finds the path an
attacker would actually walk:

- Black box first: what's exposed with no access
- White box: confirm exploitability against the source
- Chain the weak spots into a kill chain to the crown jewels
- Prove it against a local instance where possible

**Use it for:** security review of a PR, "is this branch secure," "find the
security holes," exploit-first threat assessment.
**Not for:** systems you are not authorized to test, or patching the holes. It
reports, never fixes.

### 📐 contract-branch — does this branch break its callers?

A bundled Rust tool (`ctk`), built on tree-sitter, extracts the branch's full
public contract and diffs it against the base, flagging only real shape changes:

- Public functions, methods, types, fields, variants, and constants removed, or
  narrowed below public
- Signatures changed: a param or field type added/removed/retyped, the return
  changed, a rename
- New public surface (additive — a minor bump, not a break)
- One overall semver call (major / minor / none), each finding proved by a
  `before → after` signature
- Optional CI gate: `--fail-on major` exits non-zero on a breaking change, with a
  `--baseline` for accepted breaks

Formatter-proof: reformatting, reordering members, and body rewrites never flag —
only a real token change surfaces. Reads contracts from Rust,
TypeScript/JavaScript, Python, and Go.

**Use it for:** "is this a breaking change," "will this break callers," "what
semver bump does this PR need," public API/ABI review, a merge gate.
**Not for:** patching the break, or languages outside the four (reported
unmeasured). It reports, never fixes.

## Install

krafter is its own marketplace. Add it, then install:

```
/plugin marketplace add iaserrat/krafter
/plugin install krafter@krafter
```

## Usage

Invoke a skill **manually by name (recommended)** — explicit and deterministic
about which reviewer runs:

```
/quality-branch
/red-team-branch
/contract-branch
```

Or trigger from natural language and let Claude pick the reviewer:

```
review the code quality of this branch
red-team this PR before I merge it
is this branch a breaking change?
```

Either way, each skill operates on the current branch's diff against its base.

## Layout

```
krafter/
├── .claude-plugin/
│   ├── plugin.json              plugin manifest
│   └── marketplace.json         marketplace catalog (self-hosted)
├── CHANGELOG.md                 release notes
├── LICENSE                      MIT
└── skills/
    ├── quality-branch/          cqt, the Rust measurement toolkit
    ├── red-team-branch/         rtk, the adversarial security toolkit
    └── contract-branch/         ctk, the breaking-change toolkit
```

## License

[MIT](LICENSE) © Ismael Serratos
