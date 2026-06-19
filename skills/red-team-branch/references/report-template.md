# Report Template & Severity Rubric

The report is the product, and it is an **offensive** product: it leads with what an attacker can
achieve, then proves it. It must let a busy engineer triage in 30 seconds (attack narrative + kill
chains + severity table), let the owner of each finding fix it without a follow-up question (exploit +
trace + recommendation + education), and **close with a clear, actionable fix plan** that orders the
remediation by what blocks the merge. Write it to `red-team-<branch>-<YYYY-MM-DD>.md` in the repo
root. Do not commit it and do not edit application code — this is an assessment.

## Severity rubric

Rank by **impact × likelihood**, then apply **domain weighting**: weight up any bug that touches the
crown jewels you identified in Phase 0a (the data or actions whose compromise is a reportable,
regulated, or business-ending event for *this* application). In a system handling such assets, a bug
that exposes them is at least High even if exploitation is fiddly, because the consequence is severe.
Name the relevant asset in the finding so the weighting is transparent.

| Severity | Bar |
|----------|-----|
| **Critical** | Bulk exposure of the crown jewels, mass IDOR across users, auth bypass to admin, RCE, payment/financial compromise, injection with data access. Reachable by an unauthenticated or low-privilege actor. |
| **High** | IDOR to one user's sensitive data, privilege escalation, SSRF to internal/metadata, forgeable webhook that moves money or state, stored XSS in a privileged surface, mass-assignment of an ownership/approval/amount column. |
| **Medium** | Missing rate limit on a sensitive/expensive endpoint, verbose error leaking internals, mass-assignment with limited blast radius, missing privileged-session guard on a sensitive mutation, missing audit on a regulated action. |
| **Low** | Info disclosure (versions, 403-vs-404 leakage), missing security header, weak-but-not-broken control with defense-in-depth value. |
| **Info** | Hardening suggestions, observations, things that are fine today but fragile. |

State the actor and reachability explicitly in each finding — "Critical (unauthenticated)" and
"Critical (any authenticated user)" are both Critical but tell very different stories.

**Chains escalate.** Rate a kill chain by its *objective*, not its weakest link. Several Medium
findings that compose into account takeover or bulk exfiltration are a **Critical chain** — rate it
Critical and cross-reference the member findings. Do not let a real breach hide as three "mediums."

## Report structure

Use this exact skeleton:

```markdown
# Red-Team Assessment — <branch> vs <base>

**Date**: <YYYY-MM-DD> · **Commit**: <sha> · **Reviewer**: red-team-branch skill
**Target**: <repo> · **Stack**: <languages/frameworks discovered in Phase 0a>
**Scope**: <one line — what the branch does + which entry points were in scope>
**Methodology**: black box (attacker hypotheses) -> white box (taint confirmation) -> dynamic (live proof on a local instance)

## Executive summary
- <Lead with the attack narrative: as <lowest-priv actor>, an attacker can <worst achievable outcome>
  by <one-line path>. Then 2-4 more bullets a non-security engineer understands, worst first.>
- Verdict: <Block / Fix-before-merge / Merge-with-follow-ups / No security-relevant findings>

## Kill chains
<The offensive headline. One block per chain — an ordered path from initial access to objective, each
step naming the finding that enables it. Rate by objective. Omit only if no finding chains.>

> **Chain A — <objective, e.g. "any user → full records table">** · Critical (any user)
> 1. <step — what the attacker does> → enabled by Finding #<n> (<class>)
> 2. <step> → enabled by Finding #<m> (<class>)
> 3. <objective reached: what they now have>
> Proven by: <dynamic PoC / white-box trace across the member findings>

## Findings at a glance
| # | Severity | Title | Class | Actor | Chains | Confirmed by |
|---|----------|-------|-------|-------|--------|--------------|
| 1 | Critical | ... | BOLA (API1) | any user | A | dynamic (cross_user_proven) |
| 2 | Medium | ... | Mass assignment (API3) | any user | A | white box |
| 3 | Low | ... | Info disclosure (API8) | any user | A | white box |

## Dynamic proof ledger
<Mandatory gate (SKILL Phase 3). One row per CONFIRMED critical/high finding and per complete kill
chain. Every row resolves to PROVEN, COULD-NOT-REPRODUCE, or BLOCKED(named reason) — never left
NEEDS-DYNAMIC. A verdict above this line is invalid while any row is unresolved. If nothing reached
critical/high, say "no critical/high findings to prove" and move on. The **Proof tool** column has
teeth: name the exact `rtk` subcommand that produced the evidence. A row PROVEN by hand-rolled `curl`
for a class that has an `rtk` subcommand does not pass — re-prove with the probe. `curl`/breakpoint is
legitimate only for a class `rtk` does not cover, a destructive sink (breakpoint), or no-binary +
no-`cargo`; state which in the row.>

| Finding / chain | Class | Proof tool | Outcome | Evidence / blocker |
|---|---|---|---|---|
| #1 / Chain A | BOLA (API1) | `rtk sweep --compare b` | PROVEN | A read B's record (200); anon 401 — req/resp below |
| #4 | Mass assignment (API3) | `rtk bopla --field is_admin=true` | COULD-NOT-REPRODUCE | field rejected locally; prod config may differ |
| #6 | RCE (injection) | breakpoint at exec sink | BLOCKED | no local instance boots (missing service creds) |

## Findings
<one block per finding, ordered by severity — see the finding template below>

## What this branch did well
<brief — controls that correctly refuted a hypothesis. Reinforces good patterns and shows the review was adversarial, not just negative.>

## Hypotheses tested and refuted
<short list: attacks you tried that the code actually stops, with the control that stopped them. This is evidence of coverage, not filler — it tells the reader what you checked so they trust the gaps you didn't list are genuinely absent.>

## General lessons
<2-4 cross-cutting themes for the team — the education layer at the branch level, e.g. "ownership checks are inconsistent: some endpoints scope the query to the caller, others only check existence. Standardize on one.">

## Fix plan (clear, actionable)
<The closing section: the defender's punch list. Synthesize the findings and chains into an ordered
remediation plan the owner can act on without re-reading the report. Restate the one-word verdict, then
list the fixes in priority order. Each line names the finding #, the file:line, and the concrete move
(not "add authz"). Keep the offensive vocabulary — the reader is an engineer. If nothing is
exploitable, say "nothing to fix" and stop; don't manufacture work.>

- **Verdict**: <Block / Fix-before-merge / Merge-with-follow-ups / No security-relevant findings>.
- **Fix before merge**: <Finding #n (`file:line`) — concrete fix; ...> — every Critical/High and every finding in a live kill chain.
- **Follow-ups (tracked, non-blocking)**: <Finding #n — fix> — Medium/Low.
- **Accept / monitor**: <Info-level or defense-in-depth items the team may defer, with the residual risk named>.
```

## Finding template

```markdown
### <#>. <Title> — <Severity> (<actor>)

- **Class**: <e.g. BOLA / API1:2023>
- **Location**: `path/to/file:NN` (entry) -> `path/to/sink:NN` (sink)
- **Confirmed by**: black box | white box | dynamic — for dynamic, name the proof grade the toolkit emits where it applies: `dynamic (cross_user_proven via sweep)` | `dynamic (mass-assignment persisted via bopla)` | `dynamic (live PoC)`
- **Reachability**: <lowest-privileged actor who can trigger it, and the input they control>
- **Chains with**: <Finding #s this composes with, and into which kill chain — or "standalone">

**What an attacker achieves**
<Lead with the capability/impact, then the exploit, concretely — the minimal steps. If dynamic,
include the request/response or the breakpoint observation that proved it. For a fuzz finding, cite
the deterministic `--seed` and the minimized `payload_hex` repro so the team can replay the exact
anomaly. This is offense: show the breach, do not describe a missing control.>

**Why it works (the trace)**
<Follow the tainted input from entry to sink and name the control that is missing or weak, with
file:line. If dynamic-proven, say so.>

**Recommendation**
<The specific fix in this codebase. Point at an existing safe pattern here ("scope the query to the
caller the way `OtherController` does at file:line") or at the project's own secure-coding convention
if one exists. Be concrete, not generic.>

**Education**
<2-3 sentences on the vulnerability class: what it is, why it bites in this domain, the general
principle that prevents it. Written so the author learns the pattern, not just patches the instance.>
```

## Worked example

Stack-neutral — replace the file paths, framework idioms, and asset names with the real ones from the
repository you assessed. It shows the offensive shape: a kill chain on top, then a member finding
written exploit-first.

```markdown
## Kill chains

> **Chain A — any authenticated user → full records table** · Critical (any authenticated user)
> 1. Call the list endpoint, whose JSON leaks each record's owner id — surfaced automatically by `rtk sweep` as exposed/sensitive fields (the id-harvest feed) → enabled by Finding #2 (excessive data exposure).
> 2. Replay the detail endpoint substituting each harvested id; no ownership check serves every row → enabled by Finding #1 (BOLA).
> 3. Objective reached: the attacker enumerates and exfiltrates the entire records table from one low-privilege account.
> Proven by: dynamic — seeded user A read seeded user B's record (Finding #1 PoC) after harvesting B's id from the list response (Finding #2).

### 1. Any user can read any other user's record — Critical (any authenticated user)

- **Class**: BOLA / API1:2023
- **Location**: `<entry handler>:24` (entry) -> `<data layer / model query>` (sink)
- **Confirmed by**: dynamic
- **Reachability**: any authenticated user; GET with a record id in the path.
- **Chains with**: Finding #2 → Chain A (id harvest + unscoped read = bulk exfiltration).

**What an attacker achieves**
Read any user's record from an ordinary account — and, chained with #2, the whole table. Logged in as
user A, request `GET /<resource>/{id}` substituting an id belonging to user B. Proven on a local
instance: a token for seeded user A returned HTTP 200 with seeded user B's record body.

**Why it works (the trace)**
The route resolves the record by primary key with no ownership scope (`<route/binding>:88`). The
request authorization hook returns "allowed" unconditionally (`<handler>:15`) and validation only
asserts the id exists — existence, not ownership. Nothing ties the resolved row to the authenticated
principal, so any valid id is served. Ids are sequential, so enumeration dumps the table.

**Recommendation**
Scope the lookup to the authenticated principal (`currentUser.<relation>.find(id)`), or run the
project's ownership/policy check — the same one `<SiblingController>:NN` already uses on a comparable
resource.

**Education**
BOLA is the most common and most damaging API bug because authentication ("you are user A") is not
authorization ("you may read *this* record"). Frameworks resolve a record by id automatically and will
happily hand over any id you ask for unless you explicitly scope the query to the caller. With
sequential ids, one missing ownership check is not one leaked record — it is the whole table.
```

## Tone

Write as the operator who ran the attack: direct, concrete, exploit-first. State capability ("an
attacker reads any record"), not concern ("ownership may not be checked"). No softening a Critical
into "you might want to consider." But no inflation either — a missing header is Low, and calling it
Critical burns your credibility on the finding that actually matters. Every claim carries evidence
(`file:line`, a trace, or a dynamic PoC). If you have not proven exploitability, say so and mark it
NEEDS-DYNAMIC/Info — never assert a breach you cannot back, and never declare "secure" when you mean
"I found nothing in what I attacked" (list what you attacked instead).
