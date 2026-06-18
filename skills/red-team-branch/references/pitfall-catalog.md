# Pitfall Catalog

The vulnerability classes that actually land in real applications, mapped to the OWASP API Security
Top 10 (2023) with the OWASP Web Top 10 and common framework traps folded in. Use it two ways: in
**Phase 1** to generate hypotheses fast, and in **Phase 2** to know exactly what control to look for
and where it tends to be missing.

**This catalog is stack-agnostic.** The *classes* are universal; the *probes* are starting points you
adapt to the languages and frameworks you discovered in Phase 0a. Translate every grep to the
idioms of the target — route/query/exec syntax, the auth helper's name, the ORM's mass-assignment
API — and scope to the diff when you can:

```bash
REPO=$(git rev-parse --show-toplevel)
BASE=$(git -C "$REPO" symbolic-ref --quiet refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'); BASE=${BASE:-main}
# Diff-scoped probe — what the branch introduced:
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nE '<pattern>'
# Repo-wide probe — the established pattern a change should have followed:
grep -rnE '<pattern>' "$REPO/<source-dir>"
```

When a finding does not fit a class below, still report it — this catalog is the common ground, not
the ceiling. Novel logic flaws in money/state/permission flows are exactly the kind of high-value bug
a generic checklist misses.

**Classes are building blocks, not endpoints.** The worst outcomes come from *chaining* them — an
info leak (§4/§11) hands you an id that an IDOR (§1) reads, a mass-assignment write (§3) then flips an
ownership column, an SSRF (§8) reaches cloud metadata. As you map each class, note what it could feed
or be fed by; carry those into the kill-chain step (SKILL.md Phase 2.5).

## Contents

1. [BOLA / IDOR — usually the #1 risk](#1-bola--idor-api12023)
2. [Broken authentication & guard confusion](#2-broken-authentication--guard-confusion-api22023)
3. [Mass assignment & broken object-property authz](#3-mass-assignment--broken-object-property-authorization-api32023)
4. [Excessive data exposure](#4-excessive-data-exposure-api32023)
5. [Function-level authorization gaps](#5-function-level-authorization-api52023)
6. [Sensitive business-flow abuse](#6-sensitive-business-flow-abuse-api62023)
7. [Injection (SQL / NoSQL / command / template)](#7-injection)
8. [SSRF](#8-ssrf-api72023)
9. [Unrestricted resource consumption / rate limits](#9-unrestricted-resource-consumption-api42023)
10. [Webhooks & unsafe API consumption](#10-webhooks--unsafe-api-consumption-api102023)
11. [Security misconfiguration & information disclosure](#11-security-misconfiguration--information-disclosure-api82023)
12. [File upload / download & object storage](#12-file-upload--download--object-storage)
13. [Privileged-session / impersonation mutations](#13-privileged-session--impersonation-mutations)
14. [Secrets, encryption & logging leakage](#14-secrets-encryption--logging-leakage)
15. [Cross-cutting framework traps](#15-cross-cutting-framework-traps)
16. [JWT attacks — algorithm confusion, none, kid injection](#16-jwt-attacks)
17. [HTTP request smuggling — CL.TE, TE.CL, TE.TE](#17-http-request-smuggling)
18. [CORS misconfiguration — origin reflection, null, wildcard+creds](#18-cors-misconfiguration)
19. [GraphQL attacks — introspection, batching, alias bypass](#19-graphql-attacks)

---

## 1. BOLA / IDOR (API1:2023)

The single most likely and most damaging class in most apps: an authenticated user reaching a
resource id that belongs to someone else. Anything per-user — records, profiles, files, orders,
messages — is a candidate.

**Where**: any endpoint taking an id in the path/query/body — controllers, API/RPC handlers, detail
routes, download endpoints.

**Probe** (adapt the lookup idiom to the ORM/data layer):
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'route|find\(|findById|findOne|get\(:?id|where\(.?id|params\[|\$_(GET|POST)|request\.(params|query|body)'
grep -rnEi '(record|order|file|profile|message|account|invoice)_?id' "$REPO/<entry-points-dir>"
```

**Confirm**: trace the id from request to query. The control must tie the resource to the
*authenticated principal* — scope the query to the caller, or run an explicit ownership/policy check.
Look for that, not merely an existence check.

**Gotcha**: validating that a row *exists* (or even that it matches a schema) is not validating that
the caller *owns* it. An existence check in front of an unscoped lookup is an IDOR with a speed bump.

**Prove it (Phase 3)**: `rtk sweep --compare <profile>` proves cross-user access at scale and harvests the leaked id/field names.

**Recommendation**: scope the query to the authenticated principal (e.g. `currentUser.orders.find(id)`)
or run the project's ownership/policy check. Match whatever the sibling endpoints in this repo already
do.

**Education**: BOLA is #1 on the OWASP API list because authentication answers "who are you" while
authorization answers "may you touch *this* row" — frameworks give you the first for free and the
second never. With sequential or guessable ids, one missing ownership check is not one leaked record,
it is the whole table.

---

## 2. Broken authentication & guard confusion (API2:2023)

If the app has more than one auth context (public API, admin panel, service-to-service, partner API),
each is a chance to use the wrong one or none.

**Where**: route group middleware/guards, auth config, custom auth middleware, token issuance/refresh.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'auth|guard|middleware|requireLogin|authenticate|withoutMiddleware|public|anonymous'
grep -rnEi 'middleware|guard|@PreAuthorize|before_action|require_auth' "$REPO/<routes-or-controllers>"
```

**Confirm**:
- A new endpoint must run under the *intended* auth context. A mismatch can resolve the wrong
  principal type entirely (a partner token treated as an admin, say).
- A route added *outside* an authenticated group, or after something that strips middleware, may be
  fully public. Check the group nesting, not just the one line.
- Token handling: expiry/refresh, secret not committed, no acceptance of unsigned/`alg:none` tokens,
  no trusting client-supplied identity claims.

**Gotcha**: intentionally-public routers (health checks, webhooks, marketing pages) are where a
handler that *assumes* an authenticated user will treat `null`/anonymous as "trust the request."

**Recommendation**: match the guard to the context; for genuinely public endpoints derive identity
from a verified token, never from a request-supplied id.

**Education**: multi-context auth fails at the seams — not a broken login, but a route that silently
runs under the wrong identity, so a check meant for one principal type passes for another because the
resolved "user" is not the model the code assumes.

---

## 3. Mass assignment & broken object-property authorization (API3:2023)

When request input is bound wholesale into a model (`create(req.body)`, `update(params)`,
auto-bound DTOs), an attacker can set columns you never meant to expose.

**Where**: model/entity definitions (allow-list vs block-list config), and the controllers/services
that fill them.

**Probe** (find both the unguarded models and the unfiltered writes):
```bash
grep -rnEi 'guarded\s*=\s*\[\]|fillable|strict\s*=\s*false|@JsonIgnoreProperties|attr_accessible|allow_all' "$REPO/<models-dir>"
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi '\b(create|update|fill|save|assign|merge)\b.*(body|params|all\(\)|validated\(\)|input)'
```

**Confirm**: find a write that takes unfiltered request input into a model with no allow-list.
Dangerous target columns: ownership (`user_id`/`owner_id`), state/status (skip a workflow gate),
role/permission/`is_admin`, money (`price`/`amount`/`approved`), verification timestamps.

**Gotcha**: using "validated input" is only safe if the validation enumerates an allow-list. Rules
that validate a few fields, followed by a write of the *whole* request body, reintroduce the attack.

**Prove it (Phase 3)**: `rtk bopla --field <col>=<val>` writes a privileged column and reads it back to confirm it persisted.

**Recommendation**: explicit allow-list of writable fields per model; write only enumerated, validated
fields; set state/ownership columns in code, never from input.

**Education**: this is BOPLA — broken object *property*-level authorization. Object-level authz asks
"may you touch this row"; property-level asks "may you write *this column*." A block-list-by-default
binding answers "yes" to every column, including the ones that decide ownership or whether a payment
is approved.

---

## 4. Excessive data exposure (API3:2023)

The mirror image of mass assignment: a model serialized to a response that leaks fields the caller
should never see — decrypted secrets, internal flags, other users' identifiers, soft-deleted rows,
audit metadata.

**Where**: serializers/transformers/resources, and any handler returning a raw model or `toJSON()`/
`toArray()` directly.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'serialize|toJSON|toArray|to_json|res\.(json|send)\(|return .*model|makeVisible|append|select \*'
```

**Confirm**: compare the response's keys against what the role actually needs. Watch for un-hiding
hidden attributes, appending computed sensitive values, returning a full ORM object instead of a
view/DTO, and eager-loaded relations dumped wholesale.

**Gotcha**: if a field is encrypted at rest but the model attribute is serialized, the response ships
the *decrypted* value. Encryption at rest does not redact a response.

**Prove it (Phase 3)**: `rtk sweep` surfaces the leaked JSON field names returned to the caller (and flags sensitive ones).

**Recommendation**: return a dedicated view/DTO that enumerates exactly the allowed fields per role;
keep the hidden/private field set honest; never un-hide sensitive data in a list endpoint.

**Education**: clients should never be the redaction layer. If the API ships extra fields "the
frontend just ignores," an attacker reading raw JSON sees everything. The cheapest breach is the one
where the data was simply in the response all along.

---

## 5. Function-level authorization (API5:2023)

A low-privilege user invoking a higher-privilege function, or an admin/staff action reachable by a
role that should not have it.

**Where**: admin-panel actions, privileged endpoints, request authorization hooks, gate/policy
definitions.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'authorize|can\(|cannot|isAdmin|hasRole|hasPermission|@PreAuthorize|policy|gate|return true'
grep -rnEi 'Gate::before|before_action.*admin|grant.*all|superuser' "$REPO/<auth-providers>"
```

**Confirm**:
- An authorization hook that returns "allowed" unconditionally (or is never overridden) on a route
  that mutates a sensitive resource is missing authz.
- Destructive/privileged actions (refund, approve, override, delete) with no role/permission check
  default to whoever can reach the panel.
- A blanket "grant everything to role X" rule makes every permission check pass for X — fine for true
  super-admins, dangerous if a lesser role was meant to be gated by the same ability.

**Prove it (Phase 3)**: `rtk matrix --profiles <names>` tabulates methods × identities and flags a state-changing verb a lesser role can call.

**Recommendation**: enforce a role/permission check on every privileged verb; gate admin actions
explicitly rather than relying on "only staff reach this screen."

**Education**: function-level authz is about *verbs*, not *rows* — "may you call this operation at
all." Admin panels are the classic blind spot: developers assume only staff reach them, but roles
drift and "grant everything" shortcuts turn "is staff" into "can do anything."

---

## 6. Sensitive business-flow abuse (API6:2023)

Technically-authorized actions abused at scale or out of order: replaying a payment, requesting the
same thing repeatedly, jumping a workflow to "approved," draining credit/promo, skipping a
prerequisite step.

**Where**: payment/order/state-machine services, the jobs that process them, anything with a
`status`/`state` transition.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'status|state|approve|markAs|transition|attempt|retry|idempoten|refund|credit|coupon|promo|lock'
```

**Confirm**: state transitions driven by request data instead of computed server-side; money/fulfilment
operations with no idempotency key or uniqueness guard (replayable); workflow steps that do not assert
the prerequisite happened. Race two requests and check for a lock or unique constraint.

**Prove it (Phase 3)**: `rtk race --count N` fires simultaneous requests to expose the missing lock (double-spend / TOCTOU).

**Recommendation**: compute state server-side, enforce prerequisites, make money/fulfilment operations
idempotent (unique key or row lock), and rate-limit the trigger (see §9).

**Education**: each request is individually "allowed" — the vulnerability is in the *aggregate* or
*ordering*. Threat-model the flow, not the endpoint: "what if I call this 100×, replay it, or skip
step 2?" These bugs cost money and integrity, not just data.

---

## 7. Injection

Injection survives wherever code builds an interpreter string from input: SQL/NoSQL queries built by
concatenation, shell commands, template rendering, LDAP/XPath. ORMs make it rare, which makes the
surviving raw spots high-value targets.

**Where**: raw query builders, dynamic query fragments, `exec`/`system`/`spawn`, server-side template
rendering, raw deserialization.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'raw|rawQuery|execute\(|query\(|whereRaw|selectRaw|orderByRaw|exec\(|system\(|popen|child_process|eval\(|render.*\+|\$\{.*\}|sprintf|format\('
```

**Confirm**: any interpreter string where a variable is concatenated/interpolated instead of passed as
a parameter binding. Note that **identifiers** (table/column/sort names) *cannot* be parameterized —
a user-influenced sort or filter column must be allow-listed even when values are bound. For different
data stores the payload/escaping differs (SQL Server vs MySQL vs Mongo vs a shell), so confirm against
the actual engine.

**Prove it (Phase 3)**: `rtk fuzz --mutate` drives bytes at the injection point (and `rtk callback` catches a blind, out-of-band hit).

**Recommendation**: parameter bindings for all values; allow-list any dynamic identifier against a
fixed set; avoid shelling out to user-influenced strings.

**Education**: injection persists in the few places developers dropped to strings — usually for
performance, a cross-store join, or a dynamic sort — and exactly where review attention lapses because
"the ORM handles it everywhere else."

---

## 8. SSRF (API7:2023)

Any feature that fetches a URL, host, or file path derived from user input — outbound HTTP, webhooks
it calls back, headless renderers (PDF/screenshot), image proxies, importers — is an SSRF candidate.

**Where**: outbound HTTP clients, URL/host builders, file fetchers, renderers, presigned-URL/temporary
-URL generation.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'http(s)?\.(get|post|request)|fetch\(|curl|Guzzle|file_get_contents|urlopen|requests\.(get|post)|open\(.*http|webhook.*url|render.*url|presigned|temporaryUrl'
```

**Confirm**: a request value flowing into an outbound URL, host, or file path. Renderers are the
sharpest case — if the rendered URL contains attacker input they can hit internal services or the
cloud metadata endpoint (`169.254.169.254`).

**Prove it (Phase 3)**: `rtk callback --port <p>` captures the server's call-home, proving even a blind SSRF.

**Recommendation**: allow-list outbound hosts; never let request input choose host/scheme; for
renderers, build the URL server-side from ids, not from a supplied URL.

**Education**: SSRF turns the server into the attacker's proxy onto the internal network and cloud
metadata — in a cloud deployment that can mean stealing the instance's credentials, escalating a
"fetch a URL" feature into full account access.

---

## 9. Unrestricted resource consumption (API4:2023)

Missing rate limits on expensive or abusable endpoints: search, report/PDF/export generation, and
anything that sends SMS or email — the latter is toll fraud and a way to harass real people.

**Where**: route/throttle config, SMS/email clients, OTP/login-token issuance, export/generation jobs,
pagination params.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'throttle|rateLimit|rate_limit|limiter|middleware'
grep -rnEi 'sms|sendMail|notify|otp|verification.?code|per_page|pageSize|limit' "$REPO/<source-dir>" | head -40
```

**Confirm**: a new sensitive/expensive route with no throttle; an SMS/OTP/email send reachable
unauthenticated or without per-user/per-recipient limits; unbounded pagination letting one call pull
the whole table.

**Recommendation**: throttle sensitive endpoints; cap pagination; per-recipient limits on SMS/OTP/email.

**Education**: availability and cost are security properties too. An unthrottled OTP endpoint is both a
brute-force surface and a way to run up a messaging bill and harass users — the attacker spends
nothing and you pay per message.

---

## 10. Webhooks & unsafe API consumption (API10:2023)

Inbound webhooks (payments, comms, third-party events) and trust in third-party responses. A forged
webhook can move money or inject data.

**Where**: webhook receiver routes and their signature-verification middleware, third-party response
parsing.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'webhook|signature|hmac|verify|hash_equals|constant.?time|==|==='
grep -rnEi 'hash_equals|crypto\.timingSafeEqual|hmac\.compare|secure_compare' "$REPO/<middleware-dir>"
```

**Confirm**:
- Signature compared with `==`/`===`/plain string-equal instead of a constant-time compare → timing
  attack / bypass.
- A webhook route with no signature verification at all → fully forgeable.
- Trusting a webhook/third-party payload's amount/status/identity without re-validating against your
  own records.

**Prove it (Phase 3)**: `rtk sign` forges the signature to test verification; `rtk timing` checks for a non-constant-time compare.

**Recommendation**: verify every webhook signature with a constant-time comparison; treat third-party
payloads as untrusted input and reconcile against your own state.

**Education**: a webhook URL is a public, unauthenticated endpoint that performs privileged actions —
the signature *is* the authentication. A non-constant-time comparison can leak the secret byte-by-byte
through timing, and a missing check means anyone who learns the URL can fabricate a paid order.

---

## 11. Security misconfiguration & information disclosure (API8:2023)

Debug/seed routes shipped to prod, verbose exceptions leaking internals, permissive CORS, missing
security headers, secrets in config.

**Where**: route files, exception/error handlers, framework config, environment files.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'debug|dump\(|dd\(|console\.log|printStackTrace|getMessage|trace|seed|factory|env\(|cors|access-control-allow'
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'render|report|error.*json.*message|stack'
```

**Confirm**: routes not gated behind an environment check; error handlers returning the raw exception
message (which may carry a query fragment, a name, a file path); new config reading secrets from source
instead of the environment; CORS `*` combined with credentials.

**Prove it (Phase 3)**: `rtk discover` finds debug/seed/sensitive routes (`.env`, `.git`, actuator) reachable on the running app.

**Recommendation**: gate non-prod routes behind environment checks; return generic error bodies and log
detail server-side; prefer 404 over 403 where existence is sensitive.

**Education**: misconfiguration is the most common real-world breach cause because it is invisible on
the happy path — the app works perfectly while quietly emitting a stack trace with sensitive data to
anyone who triggers an error.

---

## 12. File upload / download & object storage

User files combine three bug classes: upload (what you accept), storage (where it lands, who can read
the bucket), download (who can fetch it).

**Where**: upload handlers, storage/object-store services, download/serve endpoints, file models.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'upload|store\(|putObject|->put\(|disk\(|download|temporaryUrl|presigned|mimes|mimetype|content-type|max.?size|originalName|filename'
```

**Confirm**: type/size validation present; the stored key derived from a server id, not the client
filename (path traversal via `../` or absolute keys); downloads scoped to the owner and served via
short-lived signed URLs, not public links; nothing sensitive in a public bucket.

**Recommendation**: validate type and size, generate keys server-side, private storage + signed URLs,
owner check on every download.

**Education**: "public link that never expires" is a perennial offender, and an unscoped download id is
just IDOR (§1) wearing a file extension. Sensitive documents make each of the three classes a
reportable breach.

---

## 13. Privileged-session / impersonation mutations

If staff can impersonate users (or there is any "act as" / elevated session), mutations of sensitive
resources must be blocked or specially marked during that session — otherwise an impersonation session
alters a user's data under the user's own identity, destroying the audit trail.

**Where**: impersonation/"act-as" middleware, sensitive mutation routes, audit logging.

**Probe**:
```bash
grep -rnEi 'impersonat|act.?as|sudo|assume.?identity|on.?behalf' "$REPO/<middleware-and-routes>"
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'impersonat|act.?as|(post|put|patch|delete)'
```

**Confirm**: a new state-changing route on a sensitive resource that lacks the impersonation guard its
siblings have. Compare against routes that do.

**Recommendation**: apply the project's impersonation/"act-as" write-block (or audit marker) to
sensitive mutations.

**Education**: impersonation is a support feature that intentionally borrows a user's identity. Without
a write-block it also borrows their *authority*, so an action taken "as the user" is
indistinguishable in the audit trail from the user doing it themselves — destroying both safety and
accountability.

---

## 14. Secrets, encryption & logging leakage

Two failure modes around sensitive data: logging it (tokens, auth headers, request bodies, decrypted
values), and storing new sensitive data without the protection the established pattern uses.

**Where**: logging/telemetry calls, error reporters, redaction helpers, new model fields in migrations.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'log\.|logger|console\.(log|error)|print|info\(|error\(|debug\(|report\(|sentry|datadog|breadcrumb|context\['
git -C "$REPO" diff "origin/$BASE...HEAD" -- '*migration*' | grep -nEi 'string|text|json|token|secret|password|ssn|dob|card|email|phone|address'
```

**Confirm**: logs/telemetry context containing tokens, Authorization headers, request bodies with
sensitive data, or decrypted values; new sensitive columns stored in plaintext where the established
pattern is encryption.

**Recommendation**: redact sensitive headers/fields before logging; use the project's encryption
pattern for new sensitive columns; keep sensitive data out of error-tracker breadcrumbs.

**Education**: logs and error trackers are a second, lower-guarded copy of your data — they sync to
third parties, persist long after the request, and are read by people without need-to-know. A token or
PII in a log line is a breach with a long tail.

---

## 15. Cross-cutting framework traps

Quick hits to keep in view across all phases, regardless of stack:

- **Audit coverage** — if the app keeps an audit trail, a sensitive new action that is *not* audited
  removes the forensic record. Check that mutations on regulated/sensitive resources are auditable.
- **Feature flags as security gates** — a flag that gates access must fail *closed*. A flag defaulting
  to "allow" when the flag service is unreachable is an authz bypass.
- **Async/queue payloads** — jobs serialize their args; attacker-controlled scalars passed into a job
  carry taint into async context where it is easy to miss reaching a sink.
- **Null-on-missing lookups** — a "find or null" that is then used unguarded can flip an authz check
  into a silent no-op. Trace the null path.
- **Implicit/auto route-model binding** — frameworks that resolve a record from a path id automatically
  do so with no ownership scope unless explicitly scoped; it is IDOR-by-default (§1).
- **Validation drift** — handlers that re-read the raw request after a validation layer defeat the
  layer's allow-list.
- **Auth timing/enumeration** — login/lookup endpoints that distinguish "no such user" from "wrong
  password" enable account enumeration; pair with §9 rate limits.

---

## 16. JWT attacks

JWT tokens are ubiquitous in API auth. The most common failures: accepting "none" algorithm,
confusing RS256 public keys as HS256 HMAC secrets, exploitable `kid` header injection.

**Where**: every endpoint that accepts a Bearer token. Attack the token, not the endpoint.

**Probe**:
```bash
# Look for JWT usage patterns in the diff
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'jwt|bearer|authorization.*token|alg.*RS|alg.*HS|verify.*token|decode.*jwt'
```

**Confirm**: pass each forged token to the target endpoint. A 2xx/3xx response = exploitable.
Four attacks: alg=none (remove signature), blank-secret HMAC (key=""), key-confusion (public key as
HMAC key when alg is changed from RS256→HS256), kid path traversal.

**Prove it (Phase 3)**: `rtk jwt --token <jwt> --verify-url <endpoint> [--public-key <pubkey>] [--kid-path <path>]`

**Recommendation**: restrict allowed algorithms to the one(s) you actually use; never accept "none";
keep the key used for verification separate from the one used for signing; validate kid values.

**Education**: JWT libraries historically defaulted to accepting "none" algorithm. The fix is
explicit `algorithms=[...]` in the verify call, not just importing the library.

---

## 17. HTTP request smuggling

When a front-end proxy and back-end server disagree on where one request ends and the next begins,
an attacker can smuggle a prefix that poisons the connection queue. The smuggled request executes
in the victim's context, bypassing auth and WAF rules.

**Where**: any stack with a proxy/load-balancer in front of an application server (nginx→gunicorn,
haproxy→express, CloudFront→S3, ALB→EC2).

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'proxy|nginx|haproxy|elb|alb|cloudfront|upstream|backend'
```

**Confirm**: send conflicting Content-Length/Transfer-Encoding requests via raw TCP and check if a
follow-up request reflects the smuggled canary path in its body.

**Prove it (Phase 3)**: `rtk smuggle --url <target> [--canary-path <path>]` — note this constructs the
CL.TE/TE.CL/TE.TE payloads and flags the vector without false positives, but each request uses a
separate connection, so it cannot by itself confirm a poisoned queue (that needs connection reuse across
the proxy/backend pair). A clean run means "no canary reflected here", not "not vulnerable" — confirm a
real desync by hand against the live proxy/backend. A status flip alone is treated as benign, not a hit.

**Recommendation**: ensure proxy and backend use the same HTTP parsing strategy; disable TE on the
proxy or normalize both sides; use HTTP/2 end-to-end (eliminates the ambiguity).

**Education**: smuggling exploits ambiguity in RFC 7230. The fix is removing the ambiguity, not
patching individual headers. HTTP/2 with h2c or grpc ends the entire class.

---

## 18. CORS misconfiguration

Cross-Origin Resource Sharing headers control which origins can read the response. Misconfiguration
allows arbitrary origins to read authenticated data, defeating Same-Origin Policy.

**Where**: every endpoint that returns data. The response headers are the tell, not the request.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'cors|access-control|origin|ACAO|ACAC'
```

**Confirm**: send requests with Origin: https://evil.com, Origin: null, and check the response
headers. Issues: origin reflected in ACAO, ACAO=* with ACAC=true, null origin accepted, preflight
accepting DELETE from arbitrary origins.

**Prove it (Phase 3)**: `rtk cors --url <endpoint> [--preflight] [--origin <origin>...]`

**Recommendation**: never reflect the Origin header; never use ACAO=* with credentials; explicitly
whitelist allowed origins; reject null origins in credentialed contexts.

**Education**: CORS is a relaxation of Same-Origin Policy, not a security control. Returning
`Access-Control-Allow-Origin: *` on an authenticated endpoint is equivalent to disabling SOP
for that resource.

---

## 19. GraphQL attacks

GraphQL endpoints expose a query language that bypasses REST conventions. Introspection leaks the
schema, batching enables request smuggling, aliases defeat rate limits, and GET-based queries are
CSRF-able.

**Where**: `/graphql` endpoints. Look for GraphQL libraries in dependencies.

**Probe**:
```bash
git -C "$REPO" diff "origin/$BASE...HEAD" | grep -nEi 'graphql|apollo|mercurius|gql|type.?graphql'
grep -rnE 'graphql|/graphql' "$REPO/<routes-dir>"
```

**Confirm**: send introspection query; test batching (array of queries); test alias-based query
with 50+ aliased `__typename` fields to bypass depth/cost limits; test GET with URL-encoded query
(CSRF-able).

**Prove it (Phase 3)**: `rtk gql --url <target> [--endpoint /graphql] [--introspection] [--batching] [--aliasing]`

**Recommendation**: disable introspection in production; limit query depth and cost; disable
batching or apply per-query auth; reject GET-based queries or require a custom header; use
persisted queries for production.

**Education**: GraphQL is an execution engine, not a data fetcher. The server executes arbitrary
queries the client writes. Without depth/cost limits, `{ user { posts { author { posts { ... } } } } }`
is a DoS vector. Without alias limits, 100 `__typename` aliases defeat per-field rate limiting.
