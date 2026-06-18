//! Cross-file clone detection: index every k-gram, then for each duplicated
//! region build ONE clone group containing every file that shares it (not just
//! the first pair — see region.rs). Deterministic (sorted hash iteration + a
//! covered-set so one region is reported once). v1 is cross-file only.

use crate::cmd::dup::model::CloneGroup;
use crate::cmd::dup::region::region;
use std::collections::{HashMap, HashSet};

/// A hash this common is boilerplate; skip it to bound pairing cost.
const MAX_OCC: usize = 50;

pub struct FileTokens {
    pub path: String,
    pub lang: String,
    pub lines: Vec<usize>,
    pub kg: Vec<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub file: usize,
    pub idx: usize,
}

pub fn detect(files: &[FileTokens], k: usize, min_tokens: usize, min_lines: usize) -> Vec<CloneGroup> {
    let index = build_index(files);
    let mut covered: HashSet<Pos> = HashSet::new();
    let mut groups: Vec<CloneGroup> = Vec::new();
    let mut hashes: Vec<&u64> = index.keys().collect();
    hashes.sort();
    for h in hashes {
        let occ = &index[h];
        if occ.len() > MAX_OCC {
            continue;
        }
        if let Some(g) = region(files, occ, &mut covered, k, min_tokens, min_lines) {
            groups.push(g);
        }
    }
    groups.sort_by(|p, q| {
        q.token_length
            .cmp(&p.token_length)
            .then_with(|| p.members[0].path.cmp(&q.members[0].path))
            .then_with(|| p.members[0].start_line.cmp(&q.members[0].start_line))
    });
    groups
}

fn build_index(files: &[FileTokens]) -> HashMap<u64, Vec<Pos>> {
    let mut m: HashMap<u64, Vec<Pos>> = HashMap::new();
    for (file, f) in files.iter().enumerate() {
        for (idx, h) in f.kg.iter().enumerate() {
            m.entry(*h).or_default().push(Pos { file, idx });
        }
    }
    m
}
