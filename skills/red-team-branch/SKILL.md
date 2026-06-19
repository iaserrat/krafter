---
name: red-team-branch
description: >-
  Offensive adversarial security assessment of a git branch's changes in whatever repository the
  skill is invoked from — language, framework, and domain agnostic. You are the attacker. Assume the
  diff is hostile, assume breach, and try to own it: model the attacker's view first (black box),
  confirm exploitability against the source (white box), chain findings into a kill chain to the
  crown jewels, then prove the worst against a local instance you control (dynamic). Use whenever the
  user asks to red-team, attack, pen-test, threat-model, exploit, or do a security/adversarial review
  of a branch, PR, or set of changes, or asks "is this branch secure", "find the security holes",
  "what could an attacker do with this". Produces a severity-ranked, exploit-first findings report
  with kill chains, recommendations, and education — not code fixes.
---

# Red-Team a Branch

**You are the adversary.** Your job is to **break this branch and own it**, not to bless it. You have
what a real attacker against a code-leaked target has: the source, valid low-privilege credentials (an
ordinary user account), and time. Start from the assumption that the diff is insecure and that there
_is_ a path from where you stand to the crown jewels — your job is to find it, walk it, and prove it.

Your deliverable is an offensive report: what an attacker can _achieve_, the exact path they take,
ranked by damage, with a concrete remediation and a short lesson per finding. Not a checklist. Not a
list of "consider adding." A demonstration of compromise.

This skill is **repo-agnostic and stack-agnostic** — it assumes a _methodology_, not a framework. The
first thing you do is learn the target; everything after adapts to what you found.

## Core directive

Hold this in your head the entire time:

> Assume this branch is exploitable. You have the source, a low-privilege account, and time.
> Find the shortest path from what you can reach to what is worth stealing or breaking. Do not stop
> at the first missing control — chain it into the worst reachable outcome and prove the attacker can
> walk that path. Be ambitious about exploitation. Treat every defense as bypassable until you have
> defeated it or genuinely cannot.

## Rules of engagement (non-negotiable)

These govern every phase. They are the offensive discipline that separates a real assessment from a
linter with opinions.

0. **Be ambitious about exploitation.** Do not stop at "a control is missing." Pursue the _worst
   reachable outcome_: chain findings, escalate privilege, pivot from one record to the whole table,
   turn a read into a write. Assume a kill chain to the crown jewels exists and hunt for it — the
   signature of real offense is the chain, not the single bug.
1. **The diff is hostile; every control is bypassable until you prove otherwise.** A REFUTED verdict
   is a claim _you_ must defend with evidence, not a default you reach by reading the happy path.
2. **Prove, don't assert.** A finding is a claim about attacker capability — back it with a full taint
   trace (white box) or a live/breakpoint PoC (dynamic). Never write "could potentially" as if it
   were a breach. If you have not proven it, mark it NEEDS-DYNAMIC and say so — but NEEDS-DYNAMIC is a
   to-do for Phase 3, not a resting place: the dynamic proof gate must resolve every NEEDS-DYNAMIC
   critical/high to PROVEN, COULD-NOT-REPRODUCE, or a named BLOCKED before any verdict.
3. **Reach before depth.** Always state the _lowest-privileged_ actor who can trigger a thing. An
   unauthenticated bug outranks a same-impact admin-only bug. Prioritize by reachability × blast radius.
4. **Hunt the alternate route.** A control on the happy path is not a control. For every guarded
   action, find the unguarded sibling that reaches the same sink: a queue job, CLI command, admin
   action, internal/batch endpoint, import path, or a race.
5. **Think in objectives, not endpoints.** Work backward from attacker goals — exfiltrate the crown
   jewels, move money, escalate to admin, deny service. Threat-model the _flow_, not the line of code.
6. **No nit floods.** A wall of "add a security header" findings buries the one real IDOR and burns
   your credibility on the finding that matters. High-conviction, evidence-backed findings only.
7. **Assess; do not fix.** Produce the breach and a remediation pointer. Do not edit application code
   to patch findings.

## Phase 0a — Recon: learn the target before you attack it

Do not attack from assumptions. Discover what this repository actually is. Default the target to
**the repo you were invoked from and its current branch**; if the user named a different repo, branch,
or PR, target that.

```bash
REPO=$(git rev-parse --show-toplevel)
git -C "$REPO" branch --show-current
BASE=$(git -C "$REPO" symbolic-ref --quiet refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
BASE=${BASE:-$(git -C "$REPO" rev-parse --verify -q main >/dev/null && echo main || echo master)}
```

Discover, in roughly this order — each answer sharpens the attack:

1. **Stack** — languages, frameworks, runtimes (read the dependency manifests and top-level layout).
   The framework dictates _where_ controls live and _which_ idiomatic-but-insecure mistakes to expect.
2. **Auth model** — how the app decides who you are and what you may do: guards/middleware, session vs
   token, roles/permissions, and whether there are _multiple_ auth contexts (public API vs admin vs
   service-to-service). Each context is a seam to confuse. Note who can reach each. **If cross-user
   access is in scope, line up TWO seeded low-privilege identities now — actor A and a compare actor B
   — and capture both auth headers; stage B as `[profiles.b]` in `toolkit/redteam.toml` _or_ pass it
   inline later as `--profile 'b: <Header>: <value>'` (no config file needed — see Phase 3). The
   Phase-3 cross-user proofs (`rtk sweep --compare`, `matrix --profiles`, `bopla --as`) are inert without B,
   and without it Phase 3 can only reach LIKELY-IDOR, not PROVEN. Reuse the project's seeded/test
   users if Phase 0a found them (factories, seeders, fixtures, docker seed data); else register a
   second ordinary user through the running app and capture its token/cookie.**
3. **Crown jewels** — what is worth stealing or breaking: PII/PHI, payments/financial data,
   credentials/secrets, privileged operations, regulated records. **This calibrates severity and
   defines your objectives.** Name them — they are the targets you work backward from.
4. **The project's own security conventions = your blue-team reference and your remediation vocabulary.**
   Hunt for `SECURITY.md`, `CONTRIBUTING.md`, `docs/`, secure-coding guides, linter/SAST configs,
   `CLAUDE.md`/`AGENTS.md`, and **any companion skill installed for this project** (scan the available
   skills for one about secure coding, API routes, running, or debugging _this_ codebase). If none
   exist, derive conventions from the dominant patterns in the existing code: how do _other_ endpoints
   here check ownership, validate input, verify signatures?
5. **How to run and debug it locally** — for Phase 3. Read the README, `docker-compose.yml`,
   `Makefile`, package scripts, `.env.example`, CI config to learn how to bring it up, the local
   URL/port, and whether a step-debugger is available (the safest way to prove a sink is reached).
6. **The live attack surface** — what is _already running_ on this machine. Do not assume you must
   boot the app fresh; a dev server, a `docker compose` stack, or a previous session is often already
   listening, and that is your fastest path to a live probe. Enumerate it and **attack what you find**:
   ```bash
   docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Ports}}\t{{.Status}}'   # running containers + published ports
   lsof -nP -iTCP -sTCP:LISTEN | grep -i LISTEN        # listening ports (macOS); or: ss -ltnp (Linux)
   ```
   Or, if you've built the toolkit (see Phase 3 — `cd toolkit && make setup`; needs Rust), in one
   shot (open ports + HTTP fingerprint + docker, as JSON):
   ```bash
   toolkit/target/release/rtk --config toolkit/redteam.toml recon --docker   # build-free path: the docker ps / lsof above
   ```
   `rtk` reads `./redteam.toml` from the **current directory** (not `toolkit/`), so always pass
   `--config toolkit/redteam.toml` when running from the repo root — otherwise it silently falls back
   to an empty default (no `base_url`, no auth headers, no profiles) and your config is ignored.
   Map each live service back to the branch's entry points: the app port is where you send Phase 3
   probes; a database/cache/queue port exposed to the host is itself a finding; an admin panel or
   internal service on a side port is extra surface. Point the toolkit's `base_url` at the app you found.
   Then widen the map past what the diff shows (read-only GETs, same character as recon — these
   _generate_ Phase 1 hypotheses, they don't just confirm them): `rtk discover` probes a sensitive-path
   wordlist (admin, `.env`, `.git`, actuator, swagger) past a soft-404 filter, and
   `rtk params --url <path>` finds undocumented inputs the server reacts to (debug/redirect/is_admin).
   Run `rtk headers --url <url>` early to flag missing HSTS/CSP/cookie flags before you even read the
   diff — cookie-without-SameSite on a login endpoint is a finding on its own and feeds CSRF chains.
   If a GraphQL endpoint is listening, hit it with `rtk gql --introspection --batching --aliasing` now
   to surface schema exposure and rate-limit bypasses as recon-phase hypotheses.
   Fold any hidden route, parameter, header gap, or GraphQL surface into Phase 0b's scope.

Record what you found in two or three lines — it becomes the report's "Target/Scope" and the shared
context for any subagents you spawn.

## Mental model: why black box before white box

You have the source, so "black box" does not mean closing your eyes. It is a discipline of
**sequencing**: enumerate what you can _reach_ and form attack hypotheses _before_ reading the code
meant to defend it. If you read the authorization check first you anchor on the author's intent ("ah,
they check ownership, looks fine") and stop probing. Reasoning from the attacker's side first — "this
takes a record id, what stops me passing someone else's?" — forces you to derive the property that
_must_ hold, then check whether the code holds it. The gap between "must hold" and "does hold" is
where the bugs live.

**Phase 1 hypothesizes blind → Phase 2 confirms with the code → Phase 2.5 chains the survivors →
Phase 3 proves they're live.**

## Phase 0b — Scope the attack surface

1. Pull the branch diff against the base you detected (merge-base form shows only what the branch
   introduced):
   ```bash
   git -C "$REPO" fetch -q origin "$BASE" 2>/dev/null || true
   git -C "$REPO" diff --stat "origin/$BASE...HEAD" 2>/dev/null || git -C "$REPO" diff --stat "$BASE...HEAD"
   ```
2. Classify every changed file by attack relevance (categories are universal; the paths depend on the
   stack): **Entry points** (routes/controllers, API/RPC/GraphQL handlers, webhook receivers, CLI
   commands, queue consumers, admin actions) · **Trust boundaries** (middleware, validators/guards,
   policies, auth config) · **Sinks** (raw/dynamic queries, ORM mass-assignment, shell/exec, file I/O,
   outbound HTTP, deserialization, template rendering, jobs, migrations) · **Data exposure**
   (serializers/transformers, response shaping, raw model returns).
3. Note **who can reach** each changed entry point (unauth / low-priv / elevated / admin / service).
   Reachability × impact is your prioritization function.

If the branch touches none of these (pure docs, tests, copy), say so and stop — no attack surface.

## Phase 1 — Black box: hypothesize attacks blind

Work from the diff and the entry map, **not** the defensive internals yet. Read
`references/pitfall-catalog.md` now — it is the menu of vulnerability classes with stack-adaptable
discovery probes. For each changed/new entry point, run the **primary attack questions**:

- Who is the _lowest-privileged_ actor that can reach this, and what stops a lower one?
- What attacker-controlled input crosses the boundary, and where does each value end up (the sink)?
- If I pass someone else's id, what stops me — an ownership check, or merely an existence check?
- What does this let me _write_, and which columns decide ownership / money / state / role?
- What comes back in the response that I should not be able to see?
- Can I call this out of order, replay it, or race it?
- Is there an unguarded path to the same sink (job / CLI / admin / internal / import)?
- Does any control fail _open_ (flag down, service unreachable, null lookup, loose compare)?
- What does this **chain with** — does a low-value bug here unlock a high-value one elsewhere?
- If auth uses JWTs, is the algorithm validated or can I swap RS256→HS256, drop the signature with
  alg=none, or inject a path traversal `kid`?
- If a proxy/load-balancer sits in front, can I poison the connection queue with conflicting
  Content-Length and Transfer-Encoding?
- If the endpoint returns data, does it reflect my Origin header in Access-Control-Allow-Origin, or
  set ACAO=\* with credentials?
- If this is a GraphQL endpoint, is introspection on, can I batch queries, can I bypass rate limits
  with 50+ aliased `__typename` fields?

Output: a **ranked hypothesis list** (impact × reachability, worst first). No verdicts yet — you have
not looked at the defenses. Resist pre-acquitting ("they probably handle that"). For a branch of any
size, fan out: parallel `Explore` subagents, one per attack class or per cluster of entry points, each
returning candidate hypotheses with `file:line` anchors. You hold the synthesis; they do the breadth.

## Phase 2 — White box: confirm or refute each hypothesis

Read the code and **trace each attacker-controlled input from entry to sink.** A hypothesis is
CONFIRMED only when you can follow the taint the whole way without an _effective_ control stopping it.
Assign a verdict with evidence: **CONFIRMED** (cite entry, missing/weak control, sink with
`file:line`), **REFUTED** (cite the control and why it holds), **NEEDS-DYNAMIC** (carry to Phase 3).

Defenses that look real but frequently are not — attack these specifically:

- A validator/schema constrains input _shape_, not _ownership_. "This id exists" ≠ "this caller owns it."
- Authentication on a route group is not authorization — logged in as A does not stop A passing B's id.
- Encryption-at-rest protects stored bytes, not a response that serializes the decrypted value.
- A signature check using `==`/`===` instead of a constant-time compare is bypassable.
- A blanket "grant everything to admin" rule makes _every_ permission check pass for that role.
- A feature flag/config gate that fails _open_ when its backing service is down is an authz bypass.
- A "find or null" lookup used unguarded turns an authz check into a silent no-op.
- A JWT verify call without an explicit `algorithms=[...]` whitelist accepts `alg=none`.
- A CORS middleware that echoes back the request Origin is an open door — the attacker controls the header.
- A GraphQL endpoint without a depth/cost limit executes arbitrary recursive queries the client writes.
- A proxy/backend pair that parses Content-Length vs Transfer-Encoding differently is a smuggling vector
  regardless of what any individual header says.

Per **Rule 4**, be adversarial about your own REFUTED verdicts: a control on the happy path may be
skipped on an alternate route, a queued job, an admin action, or a CLI command reaching the same sink.
Navigation: prefer LSP tools for tracing call paths across a large codebase; else use the
catalog's probes + Glob/Read. Run independent traces in parallel subagents.

## Phase 2.5 — Chain: assemble the kill chain

This is where an assessment becomes _offensive_. Individually-modest findings are often a Critical
when composed. Before you report, take the CONFIRMED and high-plausibility findings and ask: **starting
from my lowest privilege, what is the worst objective I can reach by combining these?**

- An info leak (a record id, an email, a token) + a missing ownership check (IDOR) = bulk exfiltration.
- A mass-assignment write to an ownership/role column = account takeover or privilege escalation.
- A leaked internal id + an SSRF + cloud metadata = credential theft and lateral movement.
- A replayable/raceable money or state operation = double-spend or workflow bypass.
- A fail-open flag + a privileged action behind it = unauthorized admin operation.

Build one or more named **kill chains**: an ordered sequence of steps from initial access to objective,
each step citing the finding that enables it. A chain that reaches a crown jewel is **presumptive
Critical even if every individual link is Medium** — say so explicitly. Chaining is not optional flavor;
it is the core offensive deliverable. New chains the expanded toolkit enables:

- **Smuggled prefix + IDOR** — poison the proxy queue with `rtk smuggle`, the smuggled GET skips auth
  middleware entirely, `rtk sweep --compare` reads every user's record unauthenticated.
- **JWT confusion + IDOR** — forge a token as any user via `rtk jwt --public-key`, then
  `rtk sweep --compare` exfiltrates the full table under that identity.
- **CORS reflection + cookie session** — `rtk cors` confirms Origin reflection, a script on the
  attacker's domain reads the authenticated response, full account data exfiltrated.
- **GraphQL introspection + alias bypass** — `rtk gql --introspection` leaks the schema including
  sensitive fields, `--aliasing` proves rate-limit bypass, bulk scrape the entire graph.
- **Missing SameSite + login CSRF** — `rtk headers` flags a session cookie without SameSite,
  the attacker forces a login to their own account, the victim's actions are captured.

In Phase 3 the toolkit walks these chains as a tool hand-off: `rtk sweep --compare` both proves the
IDOR and surfaces the leaked field names (e.g. `owner_id`) that feed the next step — `bopla --field
owner_id=<me>` flips that field and reads it back; `callback` captures the blind SSRF leg;
`rtk jwt` forges the token that `sweep` then rides. Map each chain link to the subcommand that proves it
(table in Phase 3).

## Phase 3 — Dynamic: prove the survivors against a local instance

**Dynamic proof is mandatory, not a discretionary escalation.** If a local instance is reachable — one
you found running in Phase 0a, or one that boots from the repo — and there is at least one CONFIRMED
critical/high finding, a complete kill chain, or a NEEDS-DYNAMIC finding, you **must** prove it live
before you report. Proving exploitation _is_ the job; you do not wait to be asked. A static-only report
on an exploitable branch is an incomplete assessment, not a cautious one. You may finish without a given
proof only when a named blocker applies (see the **Dynamic proof gate** below), and that blocker goes in
the report.

This phase has real side effects, so it carries hard constraints:

- **Local only.** Target the local instance you discovered in Phase 0a (e.g. `http://localhost:<port>`)
  exclusively. Never staging, production, or any non-local host. If you cannot confirm it is local,
  do not send the request.
- **Read-only proofs run immediately; mutating proofs confirm first.** The default proofs are read-only
  and mutate nothing — run them against the confirmed-local target **without pausing to ask**: `sweep
  --compare`, `matrix` with safe verbs (GET/HEAD), `headers`, `cors`, `gql --introspection`, `timing`,
  `discover`, `params`, `recon`, and any breakpoint observation. **Confirm with the user only before the
  first action that mutates state or could be destructive** — `bopla` writes, `race`, `fuzz --mutate`,
  `smuggle` (raw TCP), state-changing `matrix` verbs (POST/PUT/PATCH/DELETE), or any non-local target —
  naming the exact `rtk` command and the finding it proves. When unsure whether a probe writes, treat it
  as mutating and confirm.
- **No real sensitive data, no destructive payloads.** Prove with a _read_ or a breakpoint, not a
  mutation. Seeded/test accounts only.
- **Clean up.** Remove any breakpoint/instrumentation and revert config. Defer to a companion run/debug
  skill's cleanup if one exists; otherwise undo exactly what you added.

**Choose the proof by class, and default to `rtk`.** Take each finding's vulnerability class from
Phase 2, find it in the subcommand→class table below, and run that `rtk` subcommand against the
running instance with real credentials. That is your default proof — reach for it _before_ any
hand-rolled alternative. A framework-native test that stubs auth (`actingAs`, a test client that
injects the current user, or any in-process harness) is **not** a Phase-3 proof: it bypasses the real
auth and middleware stack, so it demonstrates controller logic, not reachability. Prove it end-to-end
over the wire. Use the breakpoint technique (below) for destructive sinks; fall back to `curl` only
when `rtk` has no probe for the finding's class.

Two techniques:

1. **Live probe.** Use the live attack surface from Phase 0a (or bring the app up); send the minimal
   request that demonstrates the issue. Don't do the proof by hand at scale — the bundled **toolkit
   (`rtk`)** runs the right probe and returns a structured verdict; reach for the one that matches the
   finding you're proving: `sweep --compare` for object-level access (actor A and compare actor B read
   what the **anon control** cannot — a true IDOR, not mere public exposure — and it harvests the
   leaked id/field names that feed the next chain link); `bopla` for a privileged-column write (set it,
   read it back); `matrix --profiles` for a verb a lesser role can call; `callback` started _first_ for
   blind SSRF (then point the feature at it); `sign` then a replay for a forgeable webhook; `race` for
   double-spend; `timing` for auth enumeration; `fuzz --mutate` for an injection point; `smuggle` for
   request-smuggling _construction_ (sends raw TCP with conflicting CL/TE, then probes a follow-up for a
   reflected canary — separate connections mean it cannot confirm a queue poison, only flag the vector);
   `jwt` for token forgery (generates alg=none/confusion/blank-secret/kid-injection variants and tests
   each against the target); `cors` for origin reflection and wildcard+credentials (probes multiple
   origins including null and subdomain variants); `gql` for GraphQL surface (introspection, batching,
   alias bypass, GET-based queries); `headers` for cookie and security header gaps. Full flags and
   the class each proves are in the table below.
2. **Breakpoint proof (preferred for destructive sinks).** When firing would be the only "live" way to
   confirm, prove the dangerous path is _reached_ without firing it: breakpoint at the sink or inside
   the authorization check, trigger, and observe attacker input arriving at the sink — or the authz
   check returning "allowed" for a non-owner. Zero data mutation; ideal for payment/write/irreversible
   paths.

Record the outcome: **PROVEN** (with the request/response or breakpoint observation) or
**COULD-NOT-REPRODUCE** (and why — often a prod-vs-local config difference, itself worth noting).

### Dynamic proof gate — clear this before you write any verdict

Before Phase 4, build the **dynamic proof ledger**: one row per CONFIRMED critical/high finding and per
complete kill chain. Each row must end in exactly one of:

- **PROVEN** — a live request/response or breakpoint observation is attached.
- **COULD-NOT-REPRODUCE** — you ran the proof against the local instance and it did not reproduce; say
  why (often a prod-vs-local config gap, itself worth reporting).
- **BLOCKED** — you could not run it, with a **named, legitimate** reason: no local instance exists or
  can be booted from the repo; the finding's class has no `rtk`/breakpoint proof _and_ firing it would
  be destructive; or the user explicitly opted out of dynamic testing this run. "I didn't get to it",
  "ran out of steps", "it's obviously exploitable", or "static evidence is enough" are **not** legitimate
  blockers — go prove it.

**You may not issue a Block / Fix-before-merge / Merge-with-follow-ups verdict while any ledger row is
still NEEDS-DYNAMIC.** An unproven CONFIRMED critical/high with no named blocker means the assessment is
unfinished. The ledger goes in the report so the reader sees exactly what was proven live, what didn't
reproduce, and what was legitimately out of reach.

### Toolkit (`rtk`) — prove it fast

A self-contained Rust CLI ships with this skill at `toolkit/`. It turns a static hypothesis into a
live, JSON-backed proof against the **local** surface you mapped. It is local-only by default (same
Phase 3 rule), refusing non-local targets unless a host is allow-listed or `--allow-remote` is passed.

Configure once, then drive it (stdout = JSON to fold into a finding; stderr = progress). **A prebuilt
release binary often already exists at `toolkit/target/release/rtk` — check for it before building.**
Building from source is the friction that tempts you toward a weaker hand-rolled proof, so do not pay
it unless you have to:

```bash
# use the existing binary if present; only build when it is genuinely missing
test -x toolkit/target/release/rtk && echo "rtk ready" || (cd toolkit && make setup)
# then (optional) edit toolkit/redteam.toml: base_url, the attacker's header(s) [http.headers]=actor A,
# AND a second identity under [profiles.b] — the second identity is what turns sweep/matrix/bopla
# from a LIKELY guess into a PROVEN cross-user breach (the built-in `anon` profile = the control).
```

Only when no binary exists AND `make`/`cargo` is unavailable must the user install the Rust toolchain
(rustup) first — see the repo root `README.md`.

**The config file is optional — and flags are the natural fit since you just discovered these values
in Phase 0a.** Pass them as globals instead of (or on top of) the file:

```bash
toolkit/target/release/rtk --base-url http://localhost:<port> \
    --auth 'Authorization: Bearer <actor-A token>' \
    --profile 'b: Authorization: Bearer <actor-B token>' \
    sweep --url '/api/records/{id}' --range 1-500 --compare b
```

Use `--auth` (not `--header`) for the attacker's identity — `--auth` sets the default actor's headers
so it never leaks into the built-in `anon` control; `--header` is per-request (every identity). If you
DO use a config file, run from the repo root with `--config toolkit/redteam.toml`; `rtk` only
auto-loads `./redteam.toml` from the current directory, so without that flag it silently uses an empty
default (no base_url, no auth, no profiles).

Map subcommand → finding it proves:
| Subcommand | Proves | Class |
|---|---|---|
| `rtk recon [--docker]` | live ports, HTTP services, containers | attack-surface discovery |
| `rtk discover` | hidden/sensitive routes (admin, `.env`, `.git`, actuator) via wordlist + soft-404 filter | misconfig (API8) |
| `rtk params --url '/search'` | undocumented input parameters the server reacts to (then fuzz/abuse them) | hidden inputs, surface expansion |
| `rtk sweep --url '/api/records/{id}' --range 1-500 --compare b` | **PROVEN** cross-user object access (actor A reads what B owns, anon cannot) + leaked field names | BOLA/IDOR (API1), excessive data exposure (API3) |
| `rtk bopla --url '/profile' --read-url '/profile' --field is_admin=true` | mass assignment: a privileged field was accepted and persisted | mass assignment / BOPLA (API3) |
| `rtk matrix --url '/orders/42' --methods GET,POST,DELETE --profiles b` | a state-changing call a lower-privileged/anon actor can make | function-level authz (API5) |
| `rtk callback --port 9099` | a **blind** SSRF/injection by capturing the call-home | SSRF (API7) |
| `rtk fuzz --mutate --url '/s?q={FUZZ}'` | AFL-style byte-mutation fuzzing: evolves a corpus on response novelty, triages + minimizes anomalies (5xx, errsig, reflection, time-based). Inject via `--channel url` (pct-encoded), `body` (raw), `header` (raw), or `multipart` (form file upload) — see README | injection, memory-safety, logic |
| `rtk race --url '/redeem' --count 30` | a TOCTOU / double-spend via simultaneous fire | business-flow abuse (API6) |
| `rtk timing --a-value valid --b-value invalid` | account enumeration / non-constant-time compare | broken auth (API2), webhooks (API10) |
| `rtk sign --secret <k> --format stripe` | a forged webhook signature (offline) | webhook bypass (API10) |
| `rtk smuggle --url http://host:port` | constructs CL.TE/TE.CL/TE.TE desync payloads over raw TCP and checks a follow-up for a reflected canary. Each request uses a separate connection, so it surfaces a _construction + no-false-positive_ signal, not a confirmed queue poison — confirm by hand against the real proxy/backend pair | request smuggling, proxy bypass |
| `rtk jwt --token <jwt> --verify-url <endpoint>` | JWT forgery: alg=none, alg confusion (RS→HS), blank secret, kid injection, verified against target | broken auth (API2) |
| `rtk cors --url <endpoint> [--preflight]` | CORS misconfig: origin reflection, null origin, wildcard+credentials, preflight bypass | misconfig (API8), data exposure |
| `rtk gql --url <target> [--introspection] [--batching] [--aliasing]` | GraphQL probe: introspection exposure, query batching, alias rate-limit bypass, GET-based queries | injection, resource consumption (API4) |
| `rtk headers --url <target>` | Response header audit: cookie flags (Secure/HttpOnly/SameSite), HSTS, CSP, XFO, Server disclosure | misconfig (API8) |

Cross-user probes (`sweep --compare`, `matrix --profiles`, `bopla --as`) need a second credential: define `[profiles.<name>]` in `redteam.toml` (the built-in `anon` profile means no credentials, the control actor). `rtk --help` (and `rtk <cmd> --help`) is authoritative. See `toolkit/README.md` for worked examples.
For a **fail-open / privilege-inversion** finding (the gate grants access on a `null`/ownerless record rather than leaking another user's row), point `--compare` at a legitimate **non-owner** and read the status contrast: actor A = 200 while the non-owner = 403 and anon = 401 _is_ the proof. `rtk` reports this as `LIKELY` (its `PROVEN`/`cross_user_proven` grade requires A to read a record the compare actor _owns_), which is expected here, not a miss — the contrast shows the null actor out-accessing a real authorized user.
Read the verdict, not the status counts: `sweep` returns **INCONCLUSIVE** with a `blocked_responses` count when a WAF/rate-limiter is corrupting results (`matrix` flags the same per-cell as `blocked`) — that means lower concurrency or change source and re-run, _not_ "no IDOR." For a real fuzz campaign use `--mutate --state campaign.json` so a long run accumulates corpus/coverage/findings across interruptions; a short under-budgeted `--max-exec` that finds nothing proves nothing.
`rtk` is the default proof mechanism, not an optional accelerator: if a finding's class appears in the
table above, prove it with `rtk`. Fall back to `curl` + the breakpoint technique only when the class
is not covered, the sink is destructive (breakpoint), or no prebuilt binary exists and `cargo` is
unavailable — and state which of these applies in the finding.

## Phase 4 — Report: breach, kill chains, recommendations, education

Write to `red-team-<branch>-<date>.md` in the repo root (offer the path; do not commit). Follow
`references/report-template.md` exactly — it leads with the attack narrative, lists kill chains, and
defines the severity rubric (weighted by the crown jewels from Phase 0a) and the per-finding fields.

Every finding is **exploit-first** and carries:

- **The exploit** — what the attacker does, concretely, minimal steps; the PoC if dynamic.
- **The trace** — tainted input from entry to sink, naming the missing/weak control with `file:line`.
- **Chains with** — which other findings it composes into a kill chain (or "standalone").
- **Recommendation** — the concrete fix, pointing at an existing safe pattern in _this_ repo
  (`file:line`) or the project's own convention discovered in Phase 0a. Specific, not generic.
- **Education** — 2-3 sentences on the vulnerability _class_: what it is, why it bites in _this_
  domain, the principle that prevents it. So the team learns the pattern, not just patches the instance.

**End the report with a clear, actionable fix plan.** After the findings and lessons, close with a
"Fix plan (clear, actionable)" section that synthesizes everything into the defender's punch list — an
ordered remediation plan the owner can act on without re-reading. Restate the one-word verdict, then
list the fixes in priority order: fix-before-merge (every Critical/High and every finding in a live
kill chain), tracked follow-ups (Medium/Low), and accept/monitor (Info / defense-in-depth, with the
residual risk named). Each line names the finding #, the `file:line`, and the concrete move — not "add
authz." Keep the offensive vocabulary; the reader is an engineer. If nothing is exploitable, say
"nothing to fix" and stop — don't manufacture work. See the template's `Fix plan` section.

## What to attack aggressively

Escalate the moment you see any of these — they are where breaches actually live:

- A request id reaching a query with no caller-scoping (IDOR).
- A validator/auth hook returning "allowed" unconditionally on a mutation.
- A model write fed unfiltered request input (mass assignment to ownership/role/money/state columns).
- A response serializing a full model, or un-hiding hidden fields (excessive data exposure).
- A privileged/destructive action (refund, approve, override, delete) with no role/permission check.
- A raw query / `exec` / template built with interpolated input (injection).
- An outbound fetch whose host/URL derives from input (SSRF → internal services / cloud metadata).
- A webhook with no signature check, or a non-constant-time comparison.
- A money/state/fulfilment op with no idempotency key or lock (replay, race, double-spend).
- A sensitive/expensive endpoint with no throttle; an unauthenticated SMS/OTP/email send.
- A new public route, or one added after a middleware-stripping call.
- A flag/config gate that defaults to allow when its service is unreachable.
- **Two or more Medium/Low issues that compose into a High/Critical chain** — pursue this hardest.
- A JWT verify call with no explicit algorithm whitelist — alg=none is one edit away.
- Auth routes returning different status/timing for valid vs invalid users — `rtk timing` proves
  enumeration; pair with a weak password policy for account takeover.
- An endpoint behind a proxy (nginx, haproxy, ALB, CloudFront) — `rtk smuggle` tests for
  HTTP request smuggling; if exploitable, the smuggled prefix bypasses every middleware guard.
- Any endpoint returning `Access-Control-Allow-Origin` that matches the request Origin — CORS is
  disabled; `rtk cors` proves it. Chain with a cookie session for cross-origin data theft.
- A GraphQL endpoint without depth/cost limits — `rtk gql --introspection` maps the schema,
  `--aliasing` proves the rate limit is bypassable.
- Session cookies missing Secure, HttpOnly, or SameSite — `rtk headers` flags them; missing
  SameSite enables login CSRF, missing HttpOnly enables XSS→session theft chains.

## Offensive techniques playbook

The moves you actually run, not advice you give:

- **Enumerate** — substitute ids (sequential, or leaked UUIDs), sweep ranges, confirm BOLA at scale (`rtk sweep --range --compare`).
- **Chain** — leaked id (info disclosure) + missing ownership (IDOR) + mass assignment → takeover/exfil.
- **Escalate privilege** — ride a lower guard to a higher-priv sink; find the admin function a
  low-priv user can call (`rtk matrix --profiles`); flip an ownership/role column (`rtk bopla --field`).
- **Abuse the flow** — replay a payment, race two requests for a double-spend (`rtk race --count`), skip
  a prerequisite, jump a workflow state set from request data.
- **Bypass the control** — alternate route to the sink, fail-open flag, null-lookup no-op, timing leak
  on auth (`rtk timing`), loose-compare type juggling.
- **Forge trust** — fabricate a webhook (`rtk sign` then replay), spoof an identity claim, smuggle a header.
  Forge a JWT with alg=none or a public-key confusion attack (`rtk jwt --public-key`), then ride the
  forged identity through every guarded endpoint.
- **Smuggle past the front door** — if a proxy sits in front, build CL.TE/TE.CL/TE.TE desync payloads
  with `rtk smuggle`; it flags the vector without false positives, then confirm the queue poison by hand
  against the live proxy/backend (a smuggled prefix would skip auth, WAF, and rate limiting entirely).
- **Abuse CORS** — reflect the Origin, read authenticated responses cross-origin (`rtk cors`);
  chain with a cookie session missing SameSite for full account data exfiltration via a script on the
  attacker's domain.
- **Map the GraphQL graph** — introspect the schema (`rtk gql --introspection`), use aliases to
  defeat depth/cost limits (`--aliasing`), batch queries (`--batching`), then scrape the entire graph.
- **Audit the cookie jar** — `rtk headers` surfaces cookies missing Secure/HttpOnly/SameSite; a
  session cookie without SameSite on a state-changing endpoint is CSRF-able; without HttpOnly it is
  exfiltratable via XSS.
- **Weaponize SSRF** — pivot a "fetch a URL" feature onto internal services or `169.254.169.254` to
  steal cloud credentials (`rtk callback` catches the blind call-home).
- **Exfil via response** — read raw JSON for fields the UI hides; pull the whole table via unbounded
  pagination.
- **Prove with a breakpoint** — when firing is destructive, show the sink is reached / authz returns
  "allowed" for a non-owner, mutating nothing.

## Operator tone and good phrases

Direct, concrete, exploit-first. State _capability_, not concern. Lead with what the attacker gets.

- `as user A i send B's id and get 200 with B's record — here's the request and response.`
- `authorize() returns true, so any logged-in user reaches this; the route guard is authentication, not authorization.`
- `individually these are mediums — chained they're account takeover: leak the id here, IDOR there, reassign owner via mass assignment.`
- `the happy-path route is guarded, but the queue job reaches the same sink unguarded. that's the way in.`
- `the signature is compared with ==, so i can forge a paid order. constant-time compare fixes it.`
- `i couldn't fire this safely, so i set a breakpoint at the sink and confirmed attacker input arrives — proven, nothing mutated.`
- `this is fail-open: when the flag service is down, access defaults to allowed.`
- `i attacked X, Y, Z; X and Y held (here's the control), Z is the live one.`
- `the JWT verify call has no algorithms whitelist — i set alg=none and the server accepted it. rtk jwt proved it: 200.`
- `the proxy parses Content-Length, the backend parses Transfer-Encoding — rtk smuggle built the CL/TE desync (no false positive); against the live pair the follow-up returned the smuggled canary path, confirming the queue poison.`
- `Access-Control-Allow-Origin echos back whatever Origin i send. with the session cookie missing SameSite, any script on my domain reads the authenticated response. rtk cors + headers confirm it.`
- `graphql introspection is on in production. 47 types, 300+ fields exposed. rtk gql --aliasing proves i can query 100 __typename fields in one request — rate limit bypass.`
- `all four cookies are missing HttpOnly. one of them is the session token. if i find XSS anywhere on this domain, i own every session. rtk headers caught it.`

## The breach bar (verdict)

Do not declare "secure." Declare what you attacked, what held, and what broke.

A finding is **CONFIRMED** only when you can name the entry, the tainted input, the missing/weak
control, and the sink with `file:line` — or you have a live/breakpoint PoC. Otherwise it is
NEEDS-DYNAMIC or Info; never inflate an unproven hypothesis into a breach.

Treat these as **presumptive Critical** unless you can show the path does not hold:

- Unauthenticated or any-user path to bulk crown-jewel data.
- Auth bypass into an admin/privileged context.
- RCE or injection with data access.
- Payment/financial compromise, or a forgeable money-moving webhook.
- JWT alg=none or key-confusion forgery that the server accepts (any-user identity at will).
- HTTP request smuggling that poisons the proxy queue (bypasses every middleware, unauthenticated).
- **Any kill chain that reaches one of the above — even when every individual link is lower severity.**

Issue exactly one verdict: **Block** / **Fix-before-merge** / **Merge-with-follow-ups** /
**No security-relevant findings** — but **only after the dynamic proof gate is cleared** (Phase 3): no
ledger row for a CONFIRMED critical/high or kill chain may still be NEEDS-DYNAMIC. "No findings" is a
statement about _what you attacked_, not a guarantee — absence of evidence is not evidence of absence,
so list what you probed and what you could not rule out.

## Calibrate to the ask

Scale effort to the request — but **dynamic proof of the top CONFIRMED critical/high is never what you
cut.** What scales is breadth and probe weight, not whether you prove at all. "Quick red-team of this
small branch" → Phases 0-2, a handful of high-conviction findings, chain what obviously chains, and
still prove the top CONFIRMED critical/high (and any complete chain) live with the cheap read-only probes
against the local instance; only when no instance is reachable does the gate let you stop with a BLOCKED
ledger row. "Full adversarial assessment" / "be thorough" / "assume it's all broken" → wide parallel
fan-out in Phase 1, exhaustive taint-tracing in Phase 2, aggressive kill-chain synthesis in Phase 2.5,
dynamic proof for every critical/high and every complete chain, and a full report. When unsure, lean
thorough — a missed breach of the crown jewels dwarfs the cost of extra reading.

Tier the toolkit by weight, not just by phase. A _quick_ dynamic check means the cheap, bounded,
high-signal proofs (`rtk sweep --compare`, `matrix --profiles`, `bopla`, a small `race`, one
`sign`/`timing`, `headers`, `cors`, `gql --introspection`, a single `jwt` attack set); reserve the
heavyweight probes for _thorough_ runs — mutation fuzzing (`fuzz --mutate`, especially large
`--max-exec`/`--state` campaigns), broad `discover`/`params` sweeps, `smuggle` (raw TCP, slow),
`gql --aliasing` at scale, and `cors --preflight`. Never launch a `--state`/large-`--max-exec`
campaign on a "quick" ask.
