//! The breaking-change candidate: a reason-coded, semver-tagged diff between a
//! symbol's base-side and head-side contract, anchored at `file:line`. Like
//! cqt's candidates these are *codes, not verdicts* — the skill judges whether
//! a flagged change is actually source-breaking for this project's callers.

use serde::Serialize;

/// What changed about a public symbol between base and head.
#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Reason {
    /// A public symbol present at base is gone at head.
    Removed,
    /// A removed symbol that was already `#[deprecated]`/`@deprecated` at base —
    /// still breaking, but it had a migration window (lower priority).
    RemovedDeprecated,
    /// Same symbol, still public, but its normalized signature differs.
    SignatureChanged,
    /// Symbol still exists but was narrowed below public (e.g. `pub` → private).
    VisibilityReduced,
    /// A public symbol present at head that was not public at base (additive).
    Added,
}

/// Semantic-version impact strings (the headline the skill cites).
pub const MAJOR: &str = "major";
pub const MINOR: &str = "minor";
pub const NONE: &str = "none";

impl Reason {
    /// The minimum semver bump this change implies in isolation.
    pub fn semver(&self) -> &'static str {
        match self {
            Reason::Added => MINOR,
            _ => MAJOR,
        }
    }

    /// True for changes that can break an existing caller.
    pub fn breaking(&self) -> bool {
        !matches!(self, Reason::Added)
    }

    /// Sort priority among breaking reasons (lower = surfaced first). A removal
    /// of a *deprecated* symbol ranks below fresh breaks — it had a warning.
    pub fn rank(&self) -> u8 {
        match self {
            Reason::Removed => 0,
            Reason::VisibilityReduced => 1,
            Reason::SignatureChanged => 2,
            Reason::RemovedDeprecated => 3,
            Reason::Added => 4,
        }
    }

    /// Stable code string (matches the serialized form), for baseline keys.
    pub fn code(&self) -> &'static str {
        match self {
            Reason::Removed => "REMOVED",
            Reason::RemovedDeprecated => "REMOVED_DEPRECATED",
            Reason::SignatureChanged => "SIGNATURE_CHANGED",
            Reason::VisibilityReduced => "VISIBILITY_REDUCED",
            Reason::Added => "ADDED",
        }
    }
}

/// One reason-coded candidate finding, a self-contained evidence bundle.
#[derive(Serialize, Clone)]
pub struct Candidate {
    pub path: String,
    pub symbol: String,
    pub kind: String,
    /// Anchor line: head-side for changed/added/restricted, base-side for removed.
    pub line: usize,
    pub reason: Reason,
    pub semver: &'static str,
    pub breaking: bool,
    /// Excluded from the CI gate decision because it matched a `--baseline` entry.
    pub suppressed: bool,
    /// Base-side signature (None when the symbol is newly added).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Head-side signature (None when the symbol was removed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}
