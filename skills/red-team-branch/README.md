# red-team-branch

An **offensive** security skill for Claude Code: treat a git branch's diff as hostile, model the
attacker's view, chain findings into a kill chain to the crown jewels, and prove the worst against a
**local** instance. Repo-, stack-, and domain-agnostic — it learns the target at runtime and adapts.

It produces a severity-ranked, exploit-first findings report with kill chains, recommendations, and
education. It assesses; it does not fix.

## Layout

| Path | What it is |
|---|---|
| `SKILL.md` | the methodology Claude follows (recon → black box → white box → chain → dynamic → report) |
| `references/pitfall-catalog.md` | stack-adaptable vulnerability-class catalog (OWASP API/Web + framework traps) |
| `references/report-template.md` | report skeleton, severity rubric, kill-chain + finding templates |
| `toolkit/` | `rtk`, a self-contained Rust CLI the skill drives in the dynamic phase |
| `toolkit/README.md` | `rtk` command reference + worked examples |

## Setup

The skill itself needs no build — Claude reads `SKILL.md` and the references directly. The **toolkit**
needs a one-time build.

### Security: no prebuilt binary

This repo intentionally ships **no compiled `rtk` binary**. Offensive tooling should be built from
source you can read and trust, not delivered as an opaque executable; `toolkit/target/` is gitignored
so a binary is never committed. You build it locally, once.

### Prerequisites

- **Rust toolchain** (cargo + rustc). Install via [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- A local instance of the app under test, or a way to bring one up (the skill discovers this).

### Build the toolkit

```bash
cd toolkit
make setup        # verifies the Rust toolchain, builds the release binary, creates redteam.toml
```

`make setup` runs three steps you can also run individually:

| Target | Does |
|---|---|
| `make check` | confirms `cargo` is installed |
| `make release` | builds the optimized binary at `toolkit/target/release/rtk` |
| `make config` | copies `redteam.example.toml` → `redteam.toml` (if missing) |
| `make smoke` | builds, then self-checks the binary against a known HMAC vector |
| `make clean` | removes build artifacts |

Run `make` (or `make help`) for the full list.

### Configure

Edit `toolkit/redteam.toml`:
- `http.base_url` — where the target serves locally (discover it with `rtk recon`).
- `[http.headers]` — the **low-privilege attacker's** credentials (e.g. `Authorization`/`Cookie`).
- `[profiles.b]` — a **second user's** credentials; the keystone for cross-user proof. `sweep --compare b`,
  `matrix --profiles b`, and `bopla --as b` need it to upgrade a "LIKELY IDOR" verdict to "PROVEN".
  Configure it now, not at Phase 3 (the built-in `anon` profile is the no-credentials control).
- `[safety]` — the toolkit is **local-only by default**; non-local targets are refused unless
  allow-listed or `--allow-remote` is passed (only ever with explicit authorization).

> **Run from the repo root with `--config toolkit/redteam.toml`** (a global flag). `rtk` only
> auto-loads `./redteam.toml` from the current directory, so without the flag it silently uses an
> empty default config — no base_url, no auth, no profiles.

## Using the skill

Ask Claude to red-team a branch (e.g. *"red-team this branch"*, *"is this branch secure?"*, *"what
could an attacker do with these changes?"*). The skill will:

1. **Recon** the target — stack, auth model, crown jewels, the project's own security conventions, and
   the **live attack surface** (running servers, docker containers, exposed ports).
2. **Black box** — hypothesize attacks from the diff before reading the defenses.
3. **White box** — trace each tainted input from entry to sink; confirm or refute.
4. **Chain** — compose findings into kill chains to the crown jewels.
5. **Dynamic** — prove the survivors on the local instance, using `rtk` for the tedious probes.
6. **Report** — exploit-first findings, kill chains, recommendations, education, closing with a clear,
   actionable fix plan (verdict + prioritized remediation punch list).

See `SKILL.md` for the full methodology and `toolkit/README.md` for the `rtk` command reference.

## Safety & authorization

This is offensive tooling for **authorized** assessment of code you own or are engaged to test. The
dynamic phase and the toolkit are local-only by default, use seeded/test accounts, avoid destructive
payloads, and clean up after themselves. The local-only guard is a guardrail, not a license — do not
point any of this at systems you are not authorized to test.
