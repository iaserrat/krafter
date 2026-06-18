mod args;
mod classify;
mod fields;
mod hit;
mod ids;
mod report;

pub use args::Args;

use crate::{cmd, config::Ctx, http};
use futures::{stream, StreamExt};

const PRIVATE_SNIPPET_LEN: usize = 160;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let spec = cmd::base_spec(
        ctx,
        &args.method,
        &args.url,
        &args.headers,
        args.body.clone(),
    )?;
    let ids = ids::collect(&args)?;
    if ids.is_empty() {
        anyhow::bail!("no ids — pass --ids, --ids-file, or --range");
    }
    let clients = Clients::new(ctx, args.compare.as_deref())?;
    eprintln!(
        "[rtk] sweep: {} ids against {} (compare={})",
        ids.len(),
        spec.url,
        args.compare.as_deref().unwrap_or("none")
    );
    let hits = run_ids(&args, ctx, &ids, &spec, clients).await;
    report::emit(&args, spec.url, ids.len(), hits);
    Ok(())
}

#[derive(Clone)]
struct Clients {
    a: reqwest::Client,
    anon: reqwest::Client,
    b: Option<reqwest::Client>,
}

impl Clients {
    fn new(ctx: &Ctx, compare: Option<&str>) -> anyhow::Result<Self> {
        Ok(Self {
            a: ctx.client_for(None)?,
            anon: ctx.client_for(Some("anon"))?,
            b: compare.map(|p| ctx.client_for(Some(p))).transpose()?,
        })
    }
}

async fn run_ids(
    args: &Args,
    ctx: &Ctx,
    ids: &[String],
    spec: &http::RequestSpec,
    clients: Clients,
) -> Vec<hit::Hit> {
    stream::iter(ids.iter().cloned())
        .map(|id| run_one(id, spec.clone(), clients.clone()))
        .buffer_unordered(args.concurrency.unwrap_or(ctx.concurrency))
        .collect()
        .await
}

async fn run_one(id: String, spec: http::RequestSpec, clients: Clients) -> hit::Hit {
    let spec = spec.render("id", &id);
    let a = http::send_once(&clients.a, &spec, PRIVATE_SNIPPET_LEN).await;
    let anon = http::send_once(&clients.anon, &spec, http::NO_SNIPPET_LEN).await;
    let b = match &clients.b {
        Some(client) => Some(http::send_once(client, &spec, http::NO_SNIPPET_LEN).await),
        None => None,
    };
    classify::classify(id, a, anon, b)
}

#[cfg(test)]
mod tests;
