mod args;
mod docker;
mod fingerprint;
mod ports;
mod scan;

pub use args::Args;

use crate::{cmd, config::Ctx, http};
use serde_json::json;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    http::guard_target(
        &format!("http://{}", args.host),
        ctx.allow_remote,
        &ctx.allow_hosts,
    )?;
    let containers = if args.docker {
        docker::ps()
    } else {
        Vec::new()
    };
    let ports = ports::collect(&args, &containers);
    eprintln!(
        "[rtk] recon: scanning {} ports on {} (docker={})",
        ports.len(),
        args.host,
        args.docker
    );
    let open = scan::open_ports(&args.host, &ports, args.connect_timeout_ms, ctx.concurrency).await;
    eprintln!("[rtk] recon: {} open ports: {:?}", open.len(), open);
    let services = fingerprint::services(ctx, &args.host, &open).await?;
    cmd::emit(&json!({
        "tool": "recon", "host": args.host, "ports_scanned": ports.len(),
        "open_ports": open, "http_services": services, "docker_containers": containers,
        "hint": "feed an http_service base_url into config http.base_url, then run sweep/fuzz/race/timing against it",
    }));
    Ok(())
}
