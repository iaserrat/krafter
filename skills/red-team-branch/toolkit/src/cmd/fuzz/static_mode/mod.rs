mod classify;
mod report;

use super::{args::Args, payloads};
use crate::{cmd, config::Ctx, http};
use futures::{stream, StreamExt};

const BASELINE: &str = "rtkbaselinecanary";
const SNIPPET_LEN: usize = 240;
const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let spec = cmd::base_spec(
        ctx,
        &args.method,
        &args.url,
        &args.headers,
        args.body.clone(),
    )?;
    let payloads = payloads::load_payloads(&args)?;
    if payloads.is_empty() {
        anyhow::bail!("no payloads — pass --wordlist or --payloads <set>");
    }
    let client = http::build_client(&ctx.http)?;
    let base = http::send_once(
        &client,
        &spec.render("FUZZ", BASELINE),
        http::NO_SNIPPET_LEN,
    )
    .await;
    eprintln!(
        "[rtk] fuzz: baseline probe, then {} payloads",
        payloads.len()
    );
    let mut findings = run_payloads(&args, &payloads, &client, &spec, &base).await;
    findings.sort_by_key(|f| !f["interesting"].as_bool().unwrap_or(false));
    report::emit(spec.url, &payloads, &base, findings);
    Ok(())
}

async fn run_payloads(
    args: &Args,
    payloads: &[String],
    client: &reqwest::Client,
    spec: &http::RequestSpec,
    base: &http::Outcome,
) -> Vec<serde_json::Value> {
    stream::iter(payloads.iter().cloned())
        .map(|payload| {
            classify::payload(
                args,
                client.clone(),
                spec.render("FUZZ", &payload),
                base.clone(),
                payload,
            )
        })
        .buffer_unordered(args.concurrency.unwrap_or(DEFAULT_CONCURRENCY))
        .collect()
        .await
}

pub(super) fn snippet_len() -> usize {
    SNIPPET_LEN
}
