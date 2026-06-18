# rtk — red-team toolkit

A small, self-contained offensive Swiss-army knife that the `red-team-branch`
skill drives in **Phase 3 (dynamic)** to actually break a branch against a
**local** instance. It is not a scanner you point at the internet — it is a set
of precise probes that turn a static hypothesis into a proven breach.

## Contract (why the skill can drive it)

- **stdout** = one JSON document with the result. Parse it; fold it into a finding.
- **stderr** = human progress and warnings.
- **Local-only by default.** Non-local targets are refused unless a host is in
  `safety.allow_hosts` or `--allow-remote` is passed (only with authorization).
- **No redirects followed** (3xx is an auth/IDOR tell worth seeing).

## Build (once)

No prebuilt binary is shipped — build from source you can audit.

```bash
cd toolkit && make setup      # checks Rust, builds release, creates redteam.toml
# binary at toolkit/target/release/rtk  (or: cargo build --release)
```

`make smoke` builds and self-checks the binary. See the repo root `README.md`
for prerequisites (the Rust toolchain via rustup).

## Configure

`make config` (part of `make setup`) copies `redteam.example.toml` to
`redteam.toml`. Set `base_url` and the attacker's auth header(s). Every value is
overridable per command by a flag.

## Commands

| Command | Breaks | Vuln class |
|---|---|---|
| `recon` | discovers open ports, HTTP services, docker containers | attack-surface mapping |
| `discover` | probes a path wordlist with a soft-404 filter; flags sensitive routes | misconfig, hidden endpoints (API8) |
| `params` | finds undocumented inputs a server reacts to (reflection/diff vs baseline) | hidden parameters, surface expansion |
| `callback` | OOB server that logs hits — proves *blind* SSRF/injection | SSRF (API7), blind injection |
| `sweep` | cross-identity (A vs `--compare` vs anon) **proof** of IDOR + leaked-field surfacing | BOLA/IDOR (API1), excessive data exposure (API3) |
| `bopla` | writes a privileged field, reads back, checks it persisted | mass assignment / BOPLA (API3) |
| `matrix` | methods × identities table; flags state-changing calls a low-priv actor can make | function-level authz (API5) |
| `fuzz` | static payload list, or (`--mutate`) a real AFL-style byte-mutation fuzzer | injection, memory-safety, logic |
| `race` | fires N simultaneous requests | TOCTOU / double-spend (API6) |
| `timing` | compares latency of two inputs | auth enumeration, non-constant-time compare |
| `sign` | forges HMAC/Stripe-style webhook signatures (offline) | webhook signature bypass (API10) |

**Identities.** Cross-user probes need more than one credential. Define them in config under `[profiles.NAME]` (each replaces the default actor's auth headers); the built-in names `anon`/`none` mean "no credentials" (the control actor). `sweep --compare b`, `matrix --profiles b`, and `bopla --as b` consume them.

**No config file required.** Everything the file carries can be passed as global flags, so an agent that just discovered the values can drive the tool with zero setup:

```bash
rtk --base-url http://127.0.0.1:8080 \
    --auth 'Authorization: Bearer <actor-A>' \
    --profile 'b: Authorization: Bearer <actor-B>' \
    sweep --url '/api/records/{id}' --range 1-500 --compare b
```

Use `--auth` (not `--header`) for the attacker's identity — `--auth` sets the default actor's headers, so it never leaks into the built-in `anon` control; `--header` is per-request and applies to *every* identity (use it only for shared headers like `Content-Type`). `--profile` is repeatable and merges per key. Flags override the config file when both are present.

### Examples

```bash
RTK=./target/release/rtk

# 1. Map what's running locally (ports/docker), and discover hidden routes.
$RTK recon --docker
$RTK discover                                  # built-in sensitive-path list
$RTK discover --wordlist paths.txt --base http://127.0.0.1:8080

# 2. Prove blind SSRF: start the listener, point the target's URL field at it.
$RTK callback --port 9099 --log-file hits.jsonl
#   ...trigger the feature with http://127.0.0.1:9099/ssrf-canary-001...

# 3. PROVE IDOR — actor A reads records the compare actor owns, anon cannot.
#    Needs [profiles.b] in config (a second user's token).
$RTK sweep --url "/api/records/{id}" --range 1-500 --compare b
#    -> "cross_user_proven" class + the leaked JSON field names (the id-harvest feed).

# 3b. Mass assignment: write a privileged field, confirm it persisted.
$RTK bopla --url "/api/profile" --read-url "/api/profile" \
    --body '{"name":"x"}' --field is_admin=true --field role=admin

# 3c. Function-level authz: which methods can a lower-priv / anon actor invoke?
$RTK matrix --url "/api/orders/42" --methods GET,POST,DELETE --profiles b

# 3d. Hidden parameter discovery (find undocumented inputs to then fuzz/abuse).
$RTK params --url "/api/search"                       # query params
$RTK params --url "/api/users" --method POST --location json --body '{"name":"x"}'

# 4a. Static: fuzz a search param with SQLi probes, flag time-based blind hits.
$RTK fuzz --url "/api/search?q={FUZZ}" --payloads sqli

# 4b. Real fuzzing: AFL-style byte mutation with feedback-driven corpus + minimization.
#     Injects raw bytes at {FUZZ}; evolves on response-bucket novelty; triages + minimizes.
$RTK fuzz --mutate --url "/api/search?q={FUZZ}" --payloads sqli --seed 1 --max-exec 5000
$RTK fuzz --mutate --method POST --url "/api/import" \
    --header "Content-Type: application/json" --body '{"name":"{FUZZ}"}' --channel body
#   - deterministic: same --seed replays the exact run; each finding is minimized + has a repro.
#   - channels: url (lossy, pct-encoded) | header (raw, high-fidelity) | body (raw, high-fidelity).
#   - resumable: --state run.json persists corpus/coverage/findings; re-run to continue a long campaign.
$RTK fuzz --mutate --url "/api/search?q={FUZZ}" --max-exec 50000 --state campaign.json

# 5. Race a coupon/redeem endpoint 30x to test for double-spend.
$RTK race --url "/api/orders/{id}/redeem-credit" --method POST --count 30

# 6. Account-enumeration timing oracle.
$RTK timing --url "/api/login" --method POST \
    --body '{"email":"{VAR}","password":"x"}' \
    --header "Content-Type: application/json" \
    --a-value known@user.com --b-value nobody@nowhere.com

# 7. Forge a Stripe-style webhook signature to test the receiver.
$RTK sign --secret whsec_test --format stripe --payload-file event.json
```

## Mutation fuzzer (`--mutate`)

A compact, feedback-driven fuzzer with no external crates:

- **Byte mutation** — AFL havoc (bitflips, arithmetic, cumulative interesting values, block clone/delete/overwrite, dictionary overwrite/insert) + corpus splicing, over raw `Vec<u8>`. Injects at `{FUZZ}` in the url (pct-encoded), a header value, or the body (raw).
- **Feedback as coverage** — no instrumentation; each response is bucketed by a *stable* signature (status class, error-signature family, log-bucketed length, content class, structural body fingerprint). A never-seen bucket promotes the input into the corpus, so it evolves like grey-box fuzzing. Flaky transport/latency signals are deliberately excluded from the novelty key so jitter can't mint phantom coverage.
- **Corpus scheduling** — AFLFast FAST energy + a favored set (cheapest input per bucket).
- **Oracles** — 5xx, connection reset/timeout, latency spike (baseline+7σ, resend-confirmed), raw/encoded reflection, error signatures (SQL/stack/deser/path) absent from baseline, and differential response. One minimal repro is kept per unique anomaly.
- **Minimization** — afl-tmin (block deletion → alphabet → byte normalization), preserving the anomaly bucket; flaky buckets re-confirmed by majority.
- **Reproducible** — a hand-rolled RomuDuoJr PRNG seeded by `--seed`; the seed is printed and every finding carries `payload_hex` + a repro. Same seed ⇒ identical run. `--max-exec` is a hard request ceiling.
- **Resumable** — `--state <file>` persists the evolved corpus, coverage buckets, and findings after each round and folds them back on the next run, so a long campaign survives a kill and accumulates (coverage keys are canary-blind, so they carry across runs).

Raw response bytes are captured (capped) for the oracles but never serialized — JSON output stays binary-free.

## Safety

This is offensive tooling for **authorized** assessment of code you own or are
engaged to test. It inherits the skill's Phase 3 rules: local only, seeded/test
accounts, no destructive payloads, clean up after. The local-only guard is a
guardrail, not a license — do not point it at systems you are not authorized to
test.

## Extending

Each command is one file in `src/cmd/`. Add a subcommand by writing a module
with a `clap::Args` struct and a `run(...)`, then wiring it into `src/main.rs`
and `src/cmd/mod.rs`. Keep the stdout-JSON / stderr-progress contract.
