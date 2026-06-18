//! `rtk smuggle` — HTTP request smuggling probe via raw TCP.
//! Sends conflicting Content-Length / Transfer-Encoding requests to poison
//! the connection queue, then checks if a follow-up request is corrupted.

mod args;
mod probe;
mod raw;
mod report;
#[cfg(test)]
mod tests;

pub use args::Args;

use crate::config::Ctx;
use crate::http::guard_target;
use crate::util;

const SETTLE_MS: u64 = 100;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    guard_target(&args.url, ctx.allow_remote, &ctx.allow_hosts)?;
    let (host, port, path) = parse_target(&args.url)?;
    let host_hdr = args.host_header.as_deref().unwrap_or(&host);
    let extra = util::parse_headers(&args.headers);
    let techniques = probe::all_techniques(host_hdr, &path, &args.method, &args.canary_path, &extra);
    // Follow-up control request; its body is scanned for the smuggled canary.
    // Separate sockets cannot carry a poisoned queue, so the only sound signal is
    // the canary surfacing here — a bare status flip is benign flakiness, not desync.
    let control = format!("GET / HTTP/1.1\r\nHost: {host_hdr}\r\nConnection: close\r\n\r\n");
    let mut findings = Vec::new();
    for t in techniques {
        let probe = probe::Probe::build(t);
        eprintln!("[rtk] smuggle: testing {} ({} bytes)", probe.technique.name, probe.raw_request.len());
        let smuggle = raw::raw_request(&host, port, &probe.raw_request, args.smuggle_timeout_ms).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(SETTLE_MS)).await;
        let followup = raw::raw_request(&host, port, control.as_bytes(), args.followup_timeout_ms).await;
        findings.push(report::classify(&smuggle, &followup, &probe.technique, &args.canary_path));
    }
    report::emit(&args, &args.url, &findings);
    Ok(())
}

fn parse_target(url: &str) -> anyhow::Result<(String, u16, String)> {
    let parsed = url::Url::parse(url).map_err(|e| anyhow::anyhow!("bad url: {e}"))?;
    let host = parsed.host_str().unwrap_or("127.0.0.1").to_string();
    let port = parsed.port().unwrap_or(80);
    let path = parsed.path().to_string();
    Ok((host, port, path))
}
