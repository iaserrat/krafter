use crate::{cmd::fuzz, http};

const BASELINE_SNIPPET_LEN: usize = 200;
const WARMUP_PROBES: u64 = 30;

pub struct Warmup {
    pub base: http::Outcome,
    pub samples: Vec<u64>,
    pub exec: u64,
}

pub async fn run(
    client: &reqwest::Client,
    inj: &fuzz::injector::Injector,
    canary: &str,
    max_exec: u64,
) -> Warmup {
    let mut exec = 0;
    let base = baseline(client, inj, canary, max_exec, &mut exec).await;
    let samples = samples(client, inj, canary, max_exec, &mut exec).await;
    Warmup {
        base,
        samples,
        exec,
    }
}

async fn baseline(
    client: &reqwest::Client,
    inj: &fuzz::injector::Injector,
    canary: &str,
    max_exec: u64,
    exec: &mut u64,
) -> http::Outcome {
    if *exec >= max_exec {
        return http::Outcome::default();
    }
    *exec += 1;
    http::send_once(client, &inj.spec(canary.as_bytes()), BASELINE_SNIPPET_LEN).await
}

async fn samples(
    client: &reqwest::Client,
    inj: &fuzz::injector::Injector,
    canary: &str,
    max_exec: u64,
    exec: &mut u64,
) -> Vec<u64> {
    let mut samples = Vec::new();
    for _ in 0..max_exec.saturating_sub(*exec).min(WARMUP_PROBES) {
        *exec += 1;
        samples.push(
            http::send_once(
                client,
                &inj.spec(canary.as_bytes()),
                crate::http::NO_SNIPPET_LEN,
            )
            .await
            .latency_ms,
        );
    }
    samples
}
