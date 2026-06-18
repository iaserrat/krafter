use crate::cmd::fuzz::probe::Probe;

const CONFIRMATION_REPS: usize = 3;
const MAJORITY_FACTOR: usize = 2;
const NO_LATENCY: u64 = 0;

pub struct LatencyConfirmation {
    pub confirmed: bool,
    pub median: u64,
}

pub async fn confirm_latency(probe: &mut Probe<'_>, payload: &[u8]) -> LatencyConfirmation {
    let mut lats = Vec::new();
    for _ in 0..CONFIRMATION_REPS {
        match probe.send(payload).await {
            Some(outcome) => lats.push(outcome.latency_ms),
            None => break,
        }
    }
    if lats.is_empty() {
        return LatencyConfirmation {
            confirmed: false,
            median: NO_LATENCY,
        };
    }
    let over = lats
        .iter()
        .filter(|&&lat| lat as f64 > probe.lat_lower)
        .count();
    lats.sort_unstable();
    LatencyConfirmation {
        confirmed: over * MAJORITY_FACTOR > lats.len(),
        median: lats[lats.len() / MAJORITY_FACTOR],
    }
}
