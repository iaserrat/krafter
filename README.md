```
    ██   ██ ██████   █████  ███████ ████████ ███████ ██████
    ██  ██  ██   ██ ██   ██ ██         ██    ██      ██   ██
    █████   ██████  ███████ █████      ██    █████   ██████
    ██  ██  ██   ██ ██   ██ ██         ██    ██      ██   ██
    ██   ██ ██   ██ ██   ██ ██         ██    ███████ ██   ██

             no measurement, no finding
```

# krafter

**Two evidence-driven reviewers for the diff on your branch: one measures whether
the change made the code worse, the other attacks it like an adversary.**

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

## Install

krafter is its own marketplace. Add it, then install:

```
/plugin marketplace add iaserrat/krafter
/plugin install krafter@krafter
```

## Usage

Both skills trigger from natural language and operate on the current branch's
diff against its base:

```
review the code quality of this branch
red-team this PR before I merge it
```

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
    └── red-team-branch/         rtk, the adversarial security toolkit
```

## License

[MIT](LICENSE) © Ismael Serratos
