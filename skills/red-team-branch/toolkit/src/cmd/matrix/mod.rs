mod args;
mod identities;
mod oracle;
mod report;

pub use args::Args;

use crate::{config::Ctx, http};
use futures::{stream, StreamExt};

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let url = http::resolve_url(ctx.base_url.as_deref(), &args.url)?;
    http::guard_target(&url, ctx.allow_remote, &ctx.allow_hosts)?;
    let extra = crate::util::parse_headers(&args.headers);
    let methods = methods(&args.methods);
    let identities = identities::resolve(ctx, args.profiles.as_deref())?;
    eprintln!(
        "[rtk] matrix: {} methods x {} identities on {}",
        methods.len(),
        identities.len(),
        url
    );
    let cells = run_cells(ctx, &methods, &identities, &url, &extra, args.body.clone()).await;
    report::emit(url, methods, identities, cells);
    Ok(())
}

fn methods(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect()
}

async fn run_cells(
    ctx: &Ctx,
    methods: &[String],
    identities: &[identities::Identity],
    url: &str,
    headers: &[(String, String)],
    body: Option<String>,
) -> Vec<serde_json::Value> {
    let tasks = methods
        .iter()
        .flat_map(|m| identities.iter().map(move |i| (m.clone(), i.clone())));
    stream::iter(
        tasks.map(|(method, identity)| run_cell(method, identity, url, headers, body.clone())),
    )
    .buffer_unordered(ctx.concurrency)
    .collect()
    .await
}

async fn run_cell(
    method: String,
    identity: identities::Identity,
    url: &str,
    headers: &[(String, String)],
    body: Option<String>,
) -> serde_json::Value {
    let spec = http::RequestSpec::new(&method, url)
        .with_text_headers(headers)
        .with_body(body);
    let outcome = http::send_once(&identity.client, &spec, http::NO_SNIPPET_LEN).await;
    report::cell(method, identity.label, outcome)
}
