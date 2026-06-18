use super::{mk_seed, Session};
use crate::{cmd::cov, cmd::fuzz, config::Ctx, http};
use std::collections::{HashMap, HashSet};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const FALLBACK_SEED: u64 = 0x5eed;
const LATENCY_SIGMA_MULTIPLIER: f64 = 7.0;
const LATENCY_FLOOR_MS: f64 = 500.0;

impl Session {
    pub async fn start(args: fuzz::Args, ctx: &Ctx) -> anyhow::Result<Self> {
        let conc = args.concurrency.unwrap_or(ctx.concurrency).max(1);
        let client = http::build_client(&ctx.http)?;
        let max_exec = args.max_exec as u64;
        let inj = fuzz::injector::Injector::build(ctx, &args)?;
        inj.guard_setup()?;
        let seed_val = seed_value(args.seed);
        let mut rng = crate::cmd::mut_engine::Rng::seed(seed_val);
        let canary = format!("rtk{:08x}", rng.u32());
        let warmup = super::warmup::run(&client, &inj, &canary, max_exec).await;
        let lat_lower = crate::util::stats(&warmup.samples)
            .map(|s| (s.mean_ms + LATENCY_SIGMA_MULTIPLIER * s.stddev_ms).max(LATENCY_FLOOR_MS))
            .unwrap_or(LATENCY_FLOOR_MS);
        let corpus = initial_corpus(&args);
        let dict = fuzz::dictionary::initial(&corpus);
        let mut seen = HashSet::new();
        seen.insert(cov::novelty_key(&warmup.base, &canary, b""));
        eprintln!(
            "[rtk] fuzz(mutate): seed={seed_val} channel={} max-exec={max_exec}",
            inj.channel_name()
        );
        eprintln!(
            "[rtk] baseline status={} len={} lat_lower={:.0}ms",
            warmup.base.status, warmup.base.body_len, lat_lower
        );
        Ok(Self {
            args,
            client,
            inj,
            rng,
            seed_val,
            canary,
            base: warmup.base,
            lat_lower,
            exec: warmup.exec,
            max_exec,
            conc,
            corpus,
            dict: dict.tokens,
            dict_set: dict.seen,
            seen,
            top_rated: HashMap::new(),
            dedup: HashMap::new(),
            started: Instant::now(),
            rounds: 0,
            plateau_count: 0,
            plateau_hit: false,
        })
    }
}

fn seed_value(seed: Option<u64>) -> u64 {
    seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(FALLBACK_SEED)
    })
}

fn initial_corpus(args: &fuzz::Args) -> Vec<crate::cmd::mut_engine::Seed> {
    fuzz::payloads::load_seeds(args)
        .into_iter()
        .map(mk_seed)
        .collect()
}
