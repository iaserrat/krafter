//! Files the branch changed relative to base, with rename detection so a moved
//! function isn't double-counted as add+remove.

use crate::util::defaults::HEAD_REF;
use anyhow::Result;
use std::path::Path;

const RENAME_SIMILARITY: &str = "-M50%";
const STATUS_RENAME: char = 'R';
const STATUS_DELETE: char = 'D';

/// One changed path. `old` is the base-side path (== `new` unless renamed);
/// `new` is the HEAD-side path. Pure deletions are dropped.
pub struct ChangedFile {
    pub old: String,
    pub new: String,
}

/// Changed paths on `base...HEAD` (merge-base form), with rename pairing.
pub fn changed_files(repo: &Path, base: &str) -> Result<Vec<ChangedFile>> {
    let range = format!("{base}...{HEAD_REF}");
    let out = super::git(repo, &["diff", "--name-status", RENAME_SIMILARITY, &range])?;
    Ok(out.lines().filter_map(parse_row).collect())
}

/// Just the HEAD-side paths (for set membership: hotspot/coupling/assess).
pub fn changed_paths(repo: &Path, base: &str) -> Result<Vec<String>> {
    Ok(changed_files(repo, base)?
        .into_iter()
        .map(|c| c.new)
        .collect())
}

/// Parse a `--name-status` row: `R<score>\told\tnew`, or `<X>\tpath`.
fn parse_row(line: &str) -> Option<ChangedFile> {
    let mut cols = line.split('\t');
    let kind = cols.next()?.chars().next()?;
    if kind == STATUS_DELETE {
        return None;
    }
    if kind == STATUS_RENAME {
        let old = cols.next()?.to_string();
        let new = cols.next()?.to_string();
        return Some(ChangedFile { old, new });
    }
    let path = cols.next()?.to_string();
    Some(ChangedFile {
        old: path.clone(),
        new: path,
    })
}
