use crate::cmd::fuzz::probe::Probe;

const LATENCY_CONFIRM_REPS: usize = 3;
const STRUCTURAL_CONFIRM_REPS: usize = 1;
const MAJORITY_FACTOR: u32 = 2;

pub async fn fires_same(
    probe: &mut Probe<'_>,
    cand: &[u8],
    target: u64,
    keep_latency: bool,
) -> bool {
    let reps = if keep_latency {
        LATENCY_CONFIRM_REPS
    } else {
        STRUCTURAL_CONFIRM_REPS
    };
    let (mut hits, mut sent) = (0, 0);
    for _ in 0..reps {
        let Some(outcome) = probe.send(cand).await else {
            break;
        };
        sent += 1;
        let lat_ok = !keep_latency || outcome.latency_ms as f64 > probe.lat_lower;
        hits += (probe.bucket(&outcome, cand) == target && lat_ok) as u32;
    }
    sent > 0 && hits * MAJORITY_FACTOR > sent
}
