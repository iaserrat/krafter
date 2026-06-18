//! Tally per-file revisions and per-pair co-changes, then build coupled pairs
//! above the thresholds. degree = shared / min(revs) (default) or / mean(revs).

use crate::cmd::coupling::model::Couple;
use std::collections::HashMap;

/// Round to 4 dp so output is stable and not noisy in the last bits.
const ROUND_SCALE: f64 = 10000.0;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PairKey {
    pub a: String,
    pub b: String,
}

pub struct Tally {
    pub revs: HashMap<String, usize>,
    pub shared: HashMap<PairKey, usize>,
}

pub fn tally(commits: &[Vec<String>]) -> Tally {
    let mut revs = HashMap::new();
    let mut shared = HashMap::new();
    for files in commits {
        for f in files {
            *revs.entry(f.clone()).or_insert(0) += 1;
        }
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                // files are sorted, so a < b is canonical.
                let key = PairKey {
                    a: files[i].clone(),
                    b: files[j].clone(),
                };
                *shared.entry(key).or_insert(0) += 1;
            }
        }
    }
    Tally { revs, shared }
}

pub fn couples(t: &Tally, min_shared: usize, min_revs: usize, min_degree: f64, use_min: bool) -> Vec<Couple> {
    let mut out = Vec::new();
    for (key, &shared) in &t.shared {
        let ra = t.revs[&key.a];
        let rb = t.revs[&key.b];
        if shared < min_shared || ra < min_revs || rb < min_revs {
            continue;
        }
        let denom = if use_min {
            ra.min(rb) as f64
        } else {
            (ra + rb) as f64 / 2.0
        };
        let degree = round(shared as f64 / denom);
        if degree < min_degree {
            continue;
        }
        out.push(Couple {
            file_a: key.a.clone(),
            file_b: key.b.clone(),
            shared_commits: shared,
            revs_a: ra,
            revs_b: rb,
            degree,
            confidence_a_to_b: round(shared as f64 / ra as f64),
            confidence_b_to_a: round(shared as f64 / rb as f64),
            status: String::new(),
            anchor: String::new(),
        });
    }
    out
}

fn round(x: f64) -> f64 {
    (x * ROUND_SCALE).round() / ROUND_SCALE
}
