//! Build one clone group from a duplicated region: take the first uncovered
//! occurrence as the reference, then gather EVERY other uncovered same-language
//! file that shares the run — so an N-way clone (a block copied into many files)
//! is one group with all members, not just the first pair.

use crate::cmd::dup::detect::{FileTokens, Pos};
use crate::cmd::dup::extend::extend;
use crate::cmd::dup::model::{CloneGroup, Member};
use std::collections::HashSet;

pub fn region(files: &[FileTokens], occ: &[Pos], covered: &mut HashSet<Pos>, k: usize, min_tokens: usize, min_lines: usize) -> Option<CloneGroup> {
    let r = *occ.iter().find(|p| !covered.contains(p))?;
    let mut members: Vec<Member> = Vec::new();
    let mut token_length = 0usize;
    for &o in occ {
        if o.file == r.file || covered.contains(&o) || files[o.file].lang != files[r.file].lang {
            continue;
        }
        let run = extend(&files[r.file].kg, r.idx, &files[o.file].kg, o.idx);
        let mo = member(&files[o.file], run.b0, run.b1, k);
        if run.a1 - run.a0 + k < min_tokens || span(&mo) < min_lines {
            continue;
        }
        if members.is_empty() {
            token_length = run.a1 - run.a0 + k;
            members.push(member(&files[r.file], run.a0, run.a1, k));
            mark(covered, r.file, run.a0, run.a1);
        }
        mark(covered, o.file, run.b0, run.b1);
        members.push(mo);
    }
    if members.len() < 2 {
        return None;
    }
    members.sort_by(|x, y| x.path.cmp(&y.path).then_with(|| x.start_line.cmp(&y.start_line)));
    let line_length = members.iter().map(span).max().unwrap_or(0);
    Some(CloneGroup {
        token_length,
        line_length,
        members,
    })
}

fn member(f: &FileTokens, lo: usize, hi: usize, k: usize) -> Member {
    let last = (hi + k - 1).min(f.lines.len().saturating_sub(1));
    Member {
        path: f.path.clone(),
        start_line: f.lines.get(lo).copied().unwrap_or(0),
        end_line: f.lines.get(last).copied().unwrap_or(0),
    }
}

fn span(m: &Member) -> usize {
    m.end_line.saturating_sub(m.start_line) + 1
}

fn mark(cov: &mut HashSet<Pos>, file: usize, lo: usize, hi: usize) {
    for idx in lo..=hi {
        cov.insert(Pos { file, idx });
    }
}
