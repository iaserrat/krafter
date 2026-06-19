# Breaking-Change Catalog

A vocabulary of contract-change classes, each mapped to the **deterministic `ctk` reason code that surfaces
a candidate** and the **judgement** needed to confirm it is a real break here. The reason code makes the
candidate reproducible; you decide the true blast radius. Cite the `before → after` signature in every
finding.

Ordered roughly by how strongly the code implies a break.

---

## 1. Removed symbol

- **Code:** `REMOVED` — a symbol public at base is absent at head (deleted, or its whole file removed).
- **Judgement:** is it a real contract symbol, or `pub`/exported only for tests or siblings? Was it
  `#[deprecated]`/`@deprecated` first, giving callers a migration window? A removal with no deprecation is
  the hardest break.
- **Migration:** restore as a deprecated shim that delegates, or document the replacement and bump major.

## 2. Reduced visibility

- **Code:** `VISIBILITY_REDUCED` — the symbol still exists but was narrowed below public (`pub` → private
  or `pub(crate)`, an `export` dropped, `public` → package-private).
- **Judgement:** same as a removal from the caller's side — anyone who imported it now cannot. Confirm it
  was actually reachable externally (not re-exported elsewhere).
- **Migration:** keep it exported, or treat as an intentional removal with a major bump and a note.

## 3. Signature changed — arity

- **Code:** `SIGNATURE_CHANGED`, `before`/`after` differ by a parameter added or removed.
- **Judgement:** a new **required** parameter breaks every caller; a new parameter with a default
  (Python/TS) or a new overload may be source-compatible. A removed parameter breaks callers that passed
  it. Read the signatures, don't assume.
- **Migration:** add an overload / default, or bump major and show the new call shape.

## 4. Signature changed — types

- **Code:** `SIGNATURE_CHANGED`, a parameter or return **type** moved.
- **Judgement:** widening a parameter (accepting a supertype/trait) is often compatible; narrowing it
  breaks callers. A changed **return** type usually breaks callers that bound the old type. Type-resolution
  is beyond the tool — this is your call.
- **Migration:** keep the old type via an overload/adapter, or bump major.

## 5. Signature changed — parameter rename

- **Code:** `SIGNATURE_CHANGED`, only an identifier moved (`count` → `n`).
- **Judgement:** **breaking for keyword-argument callers** (Python `f(count=…)`, some TS object params);
  **cosmetic** for positional callers (Rust, Go, TypeScript). This is the most common false-alarm — downgrade it
  with the language reason when callers are positional.
- **Migration:** none if positional; for keyword callers, keep the old name or bump major.

## 6. Removed-but-deprecated

- **Code:** `REMOVED_DEPRECATED` — a removed symbol that was already `#[deprecated]`/`@deprecated`/
  `@Deprecated` at base.
- **Judgement:** still a break for anyone who ignored the warning, but the project gave a migration window,
  so it is lower priority than a fresh `REMOVED` (and sorts below them). Confirm the deprecation was real
  and shipped, not added in the same branch.
- **Migration:** usually none owed — the window was the migration. A team may baseline these so the gate
  does not block on them.

## 7. Additive surface

- **Code:** `ADDED` — a new public symbol (function, type, field, variant, const) not public at base.
- **Judgement:** not a break. It implies a **minor** bump (new API). Confirm it is intentional public
  surface, not an accidental `pub`/`export`.
- **Migration:** none. Credit it; it is the non-breaking half of the contract story.

---

## What ctk measures vs what to check by hand

`ctk` measures functions, methods, types, struct/Go fields, enum variants, constants, and aliases across
Rust, TypeScript/JavaScript, Python, and Go — all of the above codes apply to them uniformly (a removed
public field is `REMOVED Type::field`, a retyped field is `SIGNATURE_CHANGED`, a new variant is `ADDED`).

These are **not** resolved and should be filed as `CONSIDER` when the diff turns on them:

- **Type compatibility** — a changed parameter/field *type* is flagged `SIGNATURE_CHANGED`; whether a
  widening stays source-compatible is your call (the tool does not resolve types).
- **Known structural gaps** — Rust trait-*impl* methods (the trait defines the contract), a `pub` member
  of a private type, Go multi-name const/var specs (first name only), TS `export { x }` re-exports.
- **Unmeasured languages** — anything outside the four; the tool lists the changed file in `unmeasured`.
