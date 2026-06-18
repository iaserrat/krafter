mod args;
mod paths;
mod report;
mod request;
mod sensitive;
mod signature;

pub use args::Args;

use crate::{config::Ctx, http};
use futures::{stream, StreamExt};

// Two random non-existent paths, deliberately short and long, so the soft-404
// calibration band brackets the realistic span of a path-echoing catch-all.
const SOFT_404_PROBE_SHORT: &str = "rtk404x";
const SOFT_404_PROBE_LONG: &str = "rtk-not-here-9q8w7e6r5t-zxcvbnmasdfghjklqwertyuiop";

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let base = base_url(&args, ctx)?;
    http::guard_target(&base, ctx.allow_remote, &ctx.allow_hosts)?;
    let client = http::build_client(&ctx.http)?;
    let paths = paths::load(&args)?;
    let short = calibrate(&client, &base, SOFT_404_PROBE_SHORT).await;
    let long = calibrate(&client, &base, SOFT_404_PROBE_LONG).await;
    let soft404 = signature::Baseline::calibrate(&short, &long);
    eprintln!(
        "[rtk] discover: {} paths against {base} (soft-404 = {}xx)",
        paths.len(),
        soft404.status_class,
    );
    let routes = run_paths(&args, ctx, &client, &base, paths, soft404).await;
    report::emit(base, soft404, routes);
    Ok(())
}

async fn calibrate(client: &reqwest::Client, base: &str, path: &str) -> http::Outcome {
    http::send_once(client, &request::get(&format!("{base}/{path}")), http::NO_SNIPPET_LEN).await
}

fn base_url(args: &Args, ctx: &Ctx) -> anyhow::Result<String> {
    args.base
        .clone()
        .or_else(|| ctx.base_url.clone())
        .map(|base| base.trim_end_matches('/').to_string())
        .ok_or_else(|| anyhow::anyhow!("no base URL — pass --base or set http.base_url"))
}

async fn run_paths(
    args: &Args,
    ctx: &Ctx,
    client: &reqwest::Client,
    base: &str,
    paths: Vec<String>,
    soft404: signature::Baseline,
) -> Vec<serde_json::Value> {
    stream::iter(
        paths
            .into_iter()
            .map(|path| probe(client.clone(), base.to_string(), path, soft404)),
    )
    .buffer_unordered(args.concurrency.unwrap_or(ctx.concurrency))
    .filter_map(|x| async move { x })
    .collect()
    .await
}

async fn probe(
    client: reqwest::Client,
    base: String,
    path: String,
    soft404: signature::Baseline,
) -> Option<serde_json::Value> {
    let url = format!("{}/{}", base, path.trim_start_matches('/'));
    let outcome = http::send_once(&client, &request::get(&url), http::NO_SNIPPET_LEN).await;
    // A denied (401/403) response is always a live, protected route — never let
    // it collapse into the 404 baseline (all 4xx share one status class).
    let denied = http::is_denied(outcome.status);
    let distinct = outcome.status != http::NOT_FOUND && !soft404.indistinguishable(&outcome);
    let live = outcome.error.is_none() && (denied || distinct);
    live.then(|| report::route(path, url, outcome))
}

#[cfg(test)]
mod tests;
