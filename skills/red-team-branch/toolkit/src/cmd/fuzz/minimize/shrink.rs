use super::{normalize, test::fires_same};
use crate::cmd::fuzz::probe::Probe;

pub async fn minimize(
    probe: &mut Probe<'_>,
    payload: &[u8],
    target: u64,
    keep_latency: bool,
) -> Vec<u8> {
    let mut cur = payload.to_vec();
    delete_blocks(probe, &mut cur, target, keep_latency).await;
    normalize::alphabet(probe, &mut cur, target, keep_latency).await;
    normalize::bytes(probe, &mut cur, target, keep_latency).await;
    cur
}

async fn delete_blocks(probe: &mut Probe<'_>, cur: &mut Vec<u8>, target: u64, keep_latency: bool) {
    let mut blk = (cur.len() / 16).next_power_of_two().max(1);
    loop {
        if probe.exhausted() {
            return;
        }
        let removed = pass(probe, cur, target, keep_latency, blk).await;
        if blk == 1 && !removed {
            break;
        }
        blk = if blk == 1 { 1 } else { blk / 2 };
    }
}

async fn pass(
    probe: &mut Probe<'_>,
    cur: &mut Vec<u8>,
    target: u64,
    keep_latency: bool,
    blk: usize,
) -> bool {
    let (mut pos, mut removed) = (0, false);
    while pos < cur.len() {
        let end = (pos + blk).min(cur.len());
        let mut cand = cur.clone();
        cand.drain(pos..end);
        if !cand.is_empty() && fires_same(probe, &cand, target, keep_latency).await {
            *cur = cand;
            removed = true;
        } else {
            pos += blk;
        }
    }
    removed
}
