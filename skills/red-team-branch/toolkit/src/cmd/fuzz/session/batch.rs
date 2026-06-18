use super::{ChildCase, Session};
use crate::cmd::mut_engine;

const SKIP_PERCENT_WITH_PENDING_FAVORED: usize = 99;
const SKIP_PERCENT_DEFAULT: usize = 95;
const PERCENT_SCALE: usize = 100;

impl Session {
    pub fn mark_favored(&mut self) -> usize {
        for seed in &mut self.corpus {
            seed.favored = false;
        }
        for &idx in self.top_rated.values() {
            if idx < self.corpus.len() {
                self.corpus[idx].favored = true;
            }
        }
        self.corpus
            .iter()
            .filter(|s| s.favored && s.fuzz_level == 0)
            .count()
    }

    pub fn should_skip(&mut self, idx: usize, pending_favored: usize) -> bool {
        if self.corpus[idx].favored && self.corpus[idx].fuzz_level == 0 {
            return false;
        }
        let threshold = if pending_favored > 0 {
            SKIP_PERCENT_WITH_PENDING_FAVORED
        } else {
            SKIP_PERCENT_DEFAULT
        };
        self.rng.below(PERCENT_SCALE) < threshold
    }

    pub(super) fn children(&mut self, idx: usize) -> Vec<ChildCase> {
        let n = mut_engine::energy(&self.corpus[idx])
            .min((self.max_exec - self.exec) as u32)
            .max(1);
        let mut out = Vec::with_capacity(n as usize);
        let seed = self.corpus[idx].buf.clone();
        let barren = self.corpus[idx].barren;
        for k in 0..n as usize {
            let child = self.child(&seed, barren);
            out.push(ChildCase {
                order: k,
                spec: self.inj.spec(&child),
                payload: child,
            });
        }
        out
    }

    fn child(&mut self, seed: &[u8], barren: bool) -> Vec<u8> {
        if barren && self.corpus.len() >= 2 {
            let other = self.corpus[self.rng.below(self.corpus.len())].buf.clone();
            if let Some(mut spliced) = mut_engine::splice(seed, &other, &mut self.rng) {
                spliced = mut_engine::havoc(&spliced, &self.dict, &mut self.rng);
                return mut_engine::havoc(&spliced, &self.dict, &mut self.rng);
            }
        }
        mut_engine::havoc(seed, &self.dict, &mut self.rng)
    }
}
