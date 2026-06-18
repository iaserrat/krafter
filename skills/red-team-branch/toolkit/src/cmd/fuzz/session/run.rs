use super::Session;
use crate::cmd::fuzz;

impl Session {
    pub async fn run_loop(&mut self) {
        while self.exec < self.max_exec && !self.time_expired() {
            self.rounds += 1;
            let round_new = self.run_round().await;
            self.after_round(round_new);
            if self.plateau_hit {
                break;
            }
        }
    }

    async fn run_round(&mut self) -> u64 {
        let pending_favored = self.mark_favored();
        let snapshot = self.corpus.len();
        let mut round_new = 0;
        for idx in 0..snapshot {
            if self.exec >= self.max_exec {
                break;
            }
            if self.should_skip(idx, pending_favored) {
                continue;
            }
            let results = self.execute_seed(idx).await;
            round_new += self.fold_results(idx, &results).await;
        }
        round_new
    }

    fn after_round(&mut self, round_new: u64) {
        eprintln!(
            "[rtk] round {}: execs {}/{} unique={} anomalies={} corpus={}",
            self.rounds,
            self.exec,
            self.max_exec,
            self.seen.len(),
            self.dedup.len(),
            self.corpus.len()
        );
        self.save_state();
        self.plateau_count = if round_new == 0 {
            self.plateau_count + 1
        } else {
            0
        };
        if self.plateau_count >= self.args.plateau as u64 {
            self.plateau_hit = true;
        }
    }

    fn time_expired(&self) -> bool {
        self.args
            .max_time_secs
            .is_some_and(|limit| self.started.elapsed().as_secs() >= limit)
    }

    pub fn save_state(&self) {
        if let Some(path) = &self.args.state {
            if let Err(e) = fuzz::state::FuzzState::save(path, self) {
                eprintln!("[rtk][warn] state save failed: {e}");
            }
        }
    }
}
