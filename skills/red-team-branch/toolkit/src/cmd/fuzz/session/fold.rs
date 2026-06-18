use super::{mk_seed, BatchResult, Session};
use crate::{cmd::cov, cmd::fuzz};

impl Session {
    pub async fn fold_results(&mut self, idx: usize, results: &[BatchResult]) -> u64 {
        let base_idx = self.corpus.len();
        let (mut parent_nfuzz_add, mut seed_new, mut round_new) = (0u32, 0u64, 0u64);
        let mut new_seeds = Vec::new();
        for result in results {
            let nk = cov::novelty_key(&result.outcome, &self.canary, &result.payload);
            if self.seen.insert(nk) {
                round_new += 1;
                seed_new += 1;
                let new_idx = base_idx + new_seeds.len();
                new_seeds.push(mk_seed(result.payload.clone()));
                fuzz::dictionary::harvest(
                    &mut self.dict,
                    &mut self.dict_set,
                    &result.outcome.body_raw,
                );
                self.top_rated.insert(nk, new_idx);
            } else {
                parent_nfuzz_add = parent_nfuzz_add.saturating_add(1);
            }
            self.offer(result.payload.clone(), &result.outcome).await;
        }
        self.corpus[idx].fuzz_level += 1;
        self.corpus[idx].n_fuzz = self.corpus[idx].n_fuzz.saturating_add(parent_nfuzz_add);
        self.corpus[idx].barren = seed_new == 0;
        if self.corpus.len() < fuzz::mutate::MAX_CORPUS {
            self.corpus.extend(new_seeds);
        }
        round_new
    }

    async fn offer(&mut self, payload: Vec<u8>, outcome: &crate::http::Outcome) {
        let mask = cov::oracle_mask(outcome, &self.base, &self.canary, &payload, self.lat_lower);
        if mask == 0 {
            return;
        }
        let found_at = self.exec;
        let mut probe = fuzz::probe::Probe {
            client: &self.client,
            inj: &self.inj,
            base: &self.base,
            canary: &self.canary,
            lat_lower: self.lat_lower,
            max_exec: self.max_exec,
            exec: &mut self.exec,
        };
        fuzz::mutate::offer(
            &mut probe,
            &mut self.dedup,
            payload,
            outcome,
            mask,
            found_at,
        )
        .await;
    }
}
