use crate::{config::Ctx, http, util};
use futures::{stream, StreamExt};
use reqwest::header::{LOCATION, SERVER};
use serde_json::{json, Value};
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);
const BODY_READ_TIMEOUT: Duration = Duration::from_secs(4);

pub async fn services(ctx: &Ctx, host: &str, open: &[u16]) -> anyhow::Result<Vec<Value>> {
    let client = http::build_client(&http::HttpOpts {
        timeout: PROBE_TIMEOUT,
        insecure_tls: true,
        proxy: ctx.http.proxy.clone(),
        user_agent: ctx.http.user_agent.clone(),
        headers: ctx.http.headers.clone(),
    })?;
    Ok(stream::iter(open.iter().copied())
        .map(|port| fingerprint_port(client.clone(), host.to_string(), port))
        .buffer_unordered(ctx.concurrency)
        .filter_map(|x| async move { x })
        .collect()
        .await)
}

async fn fingerprint_port(client: reqwest::Client, host: String, port: u16) -> Option<Value> {
    match fingerprint(&client, "http", &host, port).await {
        Some(value) => Some(value),
        None => fingerprint(&client, "https", &host, port).await,
    }
}

async fn fingerprint(
    client: &reqwest::Client,
    scheme: &str,
    host: &str,
    port: u16,
) -> Option<Value> {
    let url = format!("{scheme}://{host}:{port}/");
    let resp = tokio::time::timeout(PROBE_TIMEOUT, client.get(url.as_str()).send())
        .await
        .ok()?
        .ok()?;
    let status = resp.status().as_u16();
    let server = http::header_str(resp.headers(), SERVER);
    let location = http::header_str(resp.headers(), LOCATION);
    // A body-read timeout or decode error must NOT discard an already-detected
    // service: fall back to an empty body (empty title), like send.rs read_response.
    let body = tokio::time::timeout(BODY_READ_TIMEOUT, resp.text())
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default();
    Some(
        json!({"base_url": format!("{scheme}://{host}:{port}"), "scheme": scheme, "port": port, "status": status, "server": server, "title": util::extract_title(&body), "redirect": location}),
    )
}
