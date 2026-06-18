mod args;
mod baseline;
mod detect;
mod names;
mod report;
mod request;

pub use args::Args;

use crate::{config::Ctx, http};
use futures::{stream, StreamExt};

pub const CANARY: &str = "rtkp4r4mz9";

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let client = ctx.client_for(args.as_profile.as_deref())?;
    let base = http::resolve_url(ctx.base_url.as_deref(), &args.url)?;
    http::guard_target(&base, ctx.allow_remote, &ctx.allow_hosts)?;
    let headers = crate::util::parse_headers(&args.headers);
    let names = names::load(&args)?;
    let baseline = baseline::sample(&args, &client, &base, &headers).await;
    eprintln!(
        "[rtk] params: {} candidates in {} (noisy endpoint: {})",
        names.len(),
        args.location,
        baseline.noisy
    );
    let found = run_names(&args, ctx, &client, &base, &headers, &baseline, names).await;
    report::emit(base, args.location, baseline.noisy, found);
    Ok(())
}

async fn run_names(
    args: &Args,
    ctx: &Ctx,
    client: &reqwest::Client,
    base: &str,
    headers: &[(String, String)],
    baseline: &baseline::Baseline,
    names: Vec<String>,
) -> Vec<serde_json::Value> {
    stream::iter(names.into_iter().map(|name| {
        let spec = request::spec(args, base, headers, &name);
        detect::probe(client.clone(), baseline.clone(), name, spec)
    }))
    .buffer_unordered(args.concurrency.unwrap_or(ctx.concurrency))
    .filter_map(|x| async move { x })
    .collect()
    .await
}
