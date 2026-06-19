# contract-branch

A **deterministic-evidence** breaking-change skill for Claude Code: extract a branch's public contract with
a bundled Rust tool, diff it against the base, then judge the blast radius from the signature changes.
"Will this break callers" is subjective; the skill makes the *evidence* reproducible — every finding cites
a `before → after` signature at `file:line`. Repo-agnostic across languages with explicit export semantics.

It produces a semver-ranked, signature-cited breaking-change report with migration pointers. It assesses;
it does not fix.

## Why this exists

LLM API-compatibility review drifts run-to-run and is easy to hand-wave ("this might break something").
This skill borrows the structure of `quality-branch` and `red-team-branch`: the LLM does judgement, a
deterministic Rust tool (`ctk`) provides the evidence. The discipline — *no measurement, no finding* —
kills hallucinated and inconsistent compatibility claims and makes the semver call defensible.

It is **formatter-proof by construction**: signatures are compared on a whitespace-insensitive key that
tolerates rustfmt's spacing and trailing commas, so reformatting, reordering, and body rewrites never
read as a contract change. Only a real token change — a param, a type, the return, the visibility —
surfaces. The result is a clean signal instead of a diff-noise flood.

## Layout

| Path | What it is |
|---|---|
| `SKILL.md` | the methodology Claude follows (recon → scope → measure → judge → report) |
| `references/break-catalog.md` | breaking-change classes, each mapped to the `ctk` reason code that surfaces it |
| `references/report-template.md` | report skeleton + semver verdict + scope-relative severity rubric |
| `toolkit/` | `ctk`, a self-contained Rust CLI that does the deterministic contract diffing |
| `toolkit/README.md` | `ctk` command reference + worked examples |

## Setup

The skill itself needs no build — Claude reads `SKILL.md` and the references directly. The **toolkit**
needs a one-time build.

### No prebuilt binary

This repo ships **no compiled `ctk` binary** — build from source you can audit (`toolkit/target/` is
gitignored). Extraction is built on **tree-sitter**: one uniform parse mechanism, with per-language contract
rules expressed as small walks over the parse tree (node kinds as data) rather than hand-rolled parsers —
agnostic by construction. The grammar crates are pinned to a set compatible with tree-sitter 0.24.

### Prerequisites

- **Rust toolchain** (cargo + rustc) via [rustup](https://rustup.rs). A C compiler is needed once to build
  the tree-sitter grammars.

### Build

```bash
cd toolkit
make setup        # verifies the toolchain, builds target/release/ctk
```

## Using the skill

Ask Claude whether a branch breaks the public contract (e.g. *"is this branch a breaking change?"*, *"will
this break callers?"*, *"what semver bump does this PR need?"*, *"did we remove or change any public API?"*).
The skill will:

1. **Recon** — learn the stack and decide what "public" means here (published library vs internal service
   vs app), which frames every severity.
2. **Scope** — pull the diff; note the changed source files and languages.
3. **Measure** — `ctk assess` diffs the public contract base→head into ranked, reason-coded candidates
   (`REMOVED` / `SIGNATURE_CHANGED` / `VISIBILITY_REDUCED` / `ADDED`) with an overall `semver_impact`; drill
   into a file's full surface with `ctk surface`.
4. **Judge** — decide which candidates break a real caller here (required vs defaulted param, positional vs
   keyword rename, widened vs narrowed type, reachable vs test-only); write the migration.
5. **Report** — semver-ranked, signature-cited findings with migrations, closing with a clear semver
   verdict and a migration punch list.

See `SKILL.md` for the methodology and `toolkit/README.md` for the `ctk` reference.

## Scope (honest limits)

`ctk` reads a contract from **Rust, TypeScript/JavaScript, Python, and Go** (parsed with tree-sitter) and
measures **functions, methods, types, fields, variants, constants, and aliases** — not just function
signatures. It does not resolve types (a changed param/field *type* is flagged as a signature change for
you to judge), and a few constructs are known gaps (Rust trait-impl methods, a `pub` member of a private
type, Go multi-name const specs, TS `export { x }` re-exports). Anything outside the four languages is
reported `unmeasured`. The tool surfaces candidates deterministically; the skill makes the breaking call.

## CI gate

`ctk assess` is report-only by default. Add `--fail-on major` (or `minor`) to exit non-zero as a merge
gate, and `--baseline <file>` to accept known/intentional breaks without dropping them from the report.

## Relationship to other skills

`ctk` provides the deterministic contract layer that compatibility review otherwise lacks. It complements
`quality-branch` (did the branch make the code worse?) and `red-team-branch` (is the branch exploitable?):
same evidence-first discipline, aimed at the third question — *does the branch break its callers?*
