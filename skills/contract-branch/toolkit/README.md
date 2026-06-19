# ctk — contract toolkit

A self-contained Rust CLI that produces the **deterministic measurements** the
`contract-branch` skill judges from. It extracts a file's **public contract
surface** — every public function, method, type, field, variant, constant, and
alias, each with a normalized signature and a visibility — and diffs that surface
across two git refs into ranked, reason-coded **breaking-change candidates** with
an overall semver impact and an optional CI gate. stdout is exactly one JSON
document (parse it); stderr is human progress.

It does not decide what is "truly breaking" — it measures, so the skill can rank
and the verdict stays reproducible.

## Build

```bash
make setup        # verifies cargo, builds target/release/ctk
make test         # structural gates (loc/arch/magic/positionals) then cargo test
```

Extraction is **tree-sitter**: one uniform parse mechanism, with per-language
contract rules as small walks over the parse tree. The first build compiles the
grammars (needs a C compiler) and takes a couple of minutes; afterwards cached.

## Languages

Rust · TypeScript/JavaScript (the TSX grammar covers JS/JSX) · Python · Go. A
file in any other language is reported `unmeasured` rather than measured as an
empty contract.

## Global flags

| Flag | Meaning |
|---|---|
| `--repo <path>` | repo root (default: `git rev-parse --show-toplevel` of the cwd) |
| `--base <ref>` | base ref the branch is measured against (default: `origin/HEAD`, else `main`/`master`) |

## Commands

| Command | Shows | Needs git? |
|---|---|---|
| `ctk assess [--fail-on <lvl>] [--baseline <file>]` | **the spine:** ranked breaking-change candidates + semver impact + gate | yes |
| `ctk surface --paths <f>...` | the public contract surface (symbols + signatures + visibility) of given files | no |

## Worked example

```bash
ctk --base main assess
# -> { "command":"assess","schema":"ctk.assess/v2","base":"main",
#      "semver_impact":"major",
#      "gate":{ "impact":"major","tripped":false,"suppressed":0 },
#      "candidates":[
#        { "path":"lib.rs","symbol":"Config::port","kind":"Field","line":1,
#          "reason":"SIGNATURE_CHANGED","semver":"major","breaking":true,
#          "before":"pub port: u16","after":"pub port: u32" },
#        { "path":"lib.rs","symbol":"Mode::Slow","kind":"Variant","line":2,
#          "reason":"REMOVED","semver":"major","breaking":true,"before":"Slow" },
#        { "path":"lib.rs","symbol":"helper","kind":"Function","line":9,
#          "reason":"ADDED","semver":"minor","breaking":false,"after":"pub fn helper() -> u8" } ],
#      "unmeasured":[] }
```

Each candidate is a self-contained evidence bundle: the anchor (`file:line`), the
reason code (not a verdict), the semver impact in isolation, a `breaking` flag,
and the before/after signatures as proof.

Reason codes:

- `REMOVED` — a public symbol present at base is gone at head (major).
- `REMOVED_DEPRECATED` — a removed symbol that was already `#[deprecated]` /
  `@deprecated` at base; still breaking but it had a migration window, so it
  sorts below fresh breaks (major).
- `SIGNATURE_CHANGED` — same public symbol, normalized signature differs (major).
- `VISIBILITY_REDUCED` — still present but narrowed below public (major).
- `ADDED` — a new public symbol; additive, never breaking (minor).

`semver_impact` is the headline: `major` if any candidate is breaking, else
`minor` if the branch only added surface, else `none`.

The building block (no git) — the full contract surface of given files:

```bash
ctk surface --paths src/api.rs
# -> { "command":"surface","schema":"ctk.surface/v1",
#      "files":[ { "path":"src/api.rs","lang":"rust","parse_ok":true,"vis_supported":true,
#        "symbols":[
#          { "name":"Config","kind":"Struct","visibility":"public","signature":"pub struct Config", ... },
#          { "name":"Config::host","kind":"Field","visibility":"public","signature":"pub host: String", ... } ] } ],
#      "unmeasured":0 }
```

## CI gate

`assess` is **report-only by default** (always exit 0 — the agent reads the JSON).
Opt into a hard gate with `--fail-on`:

```bash
ctk --base main assess --fail-on major     # exit 2 if any breaking change
ctk --base main assess --fail-on minor     # exit 2 on any contract change (incl. additive)
```

Accept known/intentional breaks with a baseline file (one `REASON path symbol`
key per line, `#` comments allowed). Baselined candidates are excluded from the
gate decision but still reported (`suppressed: true`), and `semver_impact` always
reflects the unfiltered truth:

```bash
printf 'REMOVED lib.rs legacy_api\n' > .ctk-baseline
ctk --base main assess --fail-on major --baseline .ctk-baseline
```

Exit codes: `0` clean (or gate not tripped), `2` gate tripped, `1` an error
(e.g. not a git repo, unreadable baseline).

## Determinism contract

- **stdout = one JSON document**; **stderr = progress.**
- **Deterministic:** tree-sitter parses deterministically; symbols and candidates
  are sorted; no wall-clock — same repo + same base → byte-identical output.
- **Formatter-proof:** signatures are compared on a whitespace-insensitive key,
  and members match by qualified name, so reformatting, reordering fields or
  variants, and body rewrites never read as a contract change.
- **Local source only:** reads the working tree and git blobs; no network.
  Oversized (>1 MiB) and binary files are skipped deterministically.

`ctk --help` and `ctk <command> --help` are authoritative.

## Scope notes (honest limits)

- **Signature-level, not type-resolved.** A changed parameter *type* shows as
  `SIGNATURE_CHANGED`; the tool flags it and the skill decides if it is truly
  source-breaking. A parameter *rename* also flags — breaking for keyword-arg
  callers (Python), cosmetic for positional ones. Codes, not verdicts.
- **Known gaps:** Rust trait-impl methods (the trait defines the contract); a
  `pub` member of a private type; Go multi-name const/var specs (first name
  only); TS re-exports (`export { x }`).
