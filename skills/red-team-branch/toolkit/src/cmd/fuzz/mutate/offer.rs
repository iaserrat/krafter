use super::confirm::confirmed_mask;
use crate::{cmd::cov, cmd::fuzz::minimize, cmd::fuzz::probe::Probe, http};
use std::collections::HashMap;

pub async fn offer(
    probe: &mut Probe<'_>,
    dedup: &mut HashMap<u64, cov::Repro>,
    payload: Vec<u8>,
    outcome: &http::Outcome,
    mask: u8,
    found_at: u64,
) {
    let confirmed = confirmed_mask(probe, &payload, outcome, mask).await;
    if confirmed.mask == 0 {
        return;
    }
    let key = probe.bucket(outcome, &payload);
    if replace_shorter(
        dedup,
        key,
        &payload,
        outcome,
        confirmed.mask,
        confirmed.latency,
    ) {
        return;
    }
    eprintln!(
        "[rtk] new anomaly bucket {key:#018x} mask={:#04x}",
        confirmed.mask
    );
    let keep_latency = confirmed.mask & cov::B_LATENCY != 0;
    let minimized = minimize::minimize(probe, &payload, key, keep_latency).await;
    dedup.insert(
        key,
        repro(
            payload,
            minimized,
            outcome,
            confirmed.mask,
            confirmed.latency,
            found_at,
        ),
    );
}

fn replace_shorter(
    dedup: &mut HashMap<u64, cov::Repro>,
    key: u64,
    payload: &[u8],
    outcome: &http::Outcome,
    mask: u8,
    latency: u64,
) -> bool {
    let Some(existing) = dedup.get(&key) else {
        return false;
    };
    if payload.len() >= existing.payload.len() {
        return true;
    }
    dedup.insert(
        key,
        repro(
            payload.to_vec(),
            payload.to_vec(),
            outcome,
            mask,
            latency,
            existing.iter,
        ),
    );
    true
}

fn repro(
    payload: Vec<u8>,
    minimized: Vec<u8>,
    outcome: &http::Outcome,
    mask: u8,
    latency: u64,
    iter: u64,
) -> cov::Repro {
    cov::Repro {
        payload,
        minimized,
        oracle_mask: mask,
        family: cov::err_family_id(outcome),
        status: outcome.status,
        body_len: outcome.body_len,
        latency_ms: latency,
        iter,
        snippet: outcome.snippet.clone(),
    }
}
