use super::test::fires_same;
use crate::{cmd::cov, cmd::fuzz::probe::Probe};

pub async fn alphabet(probe: &mut Probe<'_>, cur: &mut Vec<u8>, target: u64, keep_latency: bool) {
    let mut alphabet: Vec<u8> = cur
        .iter()
        .copied()
        .filter(|&v| v != cov::NORM_BYTE)
        .collect();
    alphabet.sort_unstable();
    alphabet.dedup();
    for value in alphabet {
        let cand = cur
            .iter()
            .map(|&c| if c == value { cov::NORM_BYTE } else { c })
            .collect::<Vec<_>>();
        if fires_same(probe, &cand, target, keep_latency).await {
            *cur = cand;
        }
    }
}

pub async fn bytes(probe: &mut Probe<'_>, cur: &mut [u8], target: u64, keep_latency: bool) {
    for i in 0..cur.len() {
        if cur[i] == cov::NORM_BYTE {
            continue;
        }
        let mut cand = cur.to_vec();
        cand[i] = cov::NORM_BYTE;
        if fires_same(probe, &cand, target, keep_latency).await {
            cur[i] = cov::NORM_BYTE;
        }
    }
}
