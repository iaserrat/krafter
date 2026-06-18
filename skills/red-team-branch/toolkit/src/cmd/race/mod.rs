//! `rtk race` — fire N identical requests at a barrier to expose TOCTOU /
//! missing-lock bugs such as double-spend, coupon reuse, or limit bypass.

mod args;
mod range;
mod report;

use crate::config::Ctx;
use crate::http;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio::sync::Barrier;

pub use args::Args;

const RESPONSE_SNIPPET_LEN: usize = 120;

pub async fn run(a: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let spec = super::base_spec(ctx, &a.method, &a.url, &a.headers, a.body)?;
    let success_range = range::parse(&a.success)?;
    let client = Arc::new(http::build_client(&ctx.http)?);
    let barrier = Arc::new(Barrier::new(a.count));
    eprintln!(
        "[rtk] race: {} simultaneous {} {}",
        a.count, spec.method, spec.url
    );

    let mut handles = Vec::with_capacity(a.count);
    for _ in 0..a.count {
        let client = client.clone();
        let barrier = barrier.clone();
        let spec = spec.clone();
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            http::send_once(&client, &spec, RESPONSE_SNIPPET_LEN).await
        }));
    }

    let mut by_status: BTreeMap<u16, usize> = BTreeMap::new();
    let mut successes = 0usize;
    // Negative control: distinct response bodies among successes = distinct real
    // effects. A locked/idempotent endpoint yields one; a double-spend many.
    let mut effects: BTreeSet<String> = BTreeSet::new();
    let mut latencies = Vec::with_capacity(a.count);
    for h in handles {
        if let Ok(o) = h.await {
            *by_status.entry(o.status).or_default() += 1;
            latencies.push(o.latency_ms);
            if success_range.contains(o.status) {
                successes += 1;
                effects.insert(o.body_sha8);
            }
        }
    }

    report::emit(report::Verdict {
        url: &spec.url,
        success_range: &a.success,
        fired: a.count,
        successes,
        distinct_successes: effects.len(),
        by_status,
        latencies: &latencies,
    });
    Ok(())
}
