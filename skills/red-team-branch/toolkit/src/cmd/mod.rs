pub mod bopla;
pub mod callback;
pub mod cors;
pub mod cov;
pub mod discover;
pub mod fuzz;
pub mod gql;
pub mod headers;
pub mod jwt;
pub mod matrix;
pub mod mut_engine;
pub mod params;
pub mod race;
pub mod recon;
pub mod sign;
pub mod smuggle;
pub mod sweep;
pub mod timing;

use crate::config::Ctx;
use crate::http::{self, RequestSpec};
use crate::util;

/// Build a base RequestSpec shared by the request-sending subcommands from
/// common flags, resolving the URL against the configured base and applying the
/// local-only guard.
pub fn base_spec(
    ctx: &Ctx,
    method: &str,
    url: &str,
    headers: &[String],
    body: Option<String>,
) -> anyhow::Result<RequestSpec> {
    let resolved = http::resolve_url(ctx.base_url.as_deref(), url)?;
    http::guard_target(&resolved, ctx.allow_remote, &ctx.allow_hosts)?;
    Ok(RequestSpec::new(method, resolved)
        .with_text_headers(&util::parse_headers(headers))
        .with_body(body))
}

/// Emit the final result document to stdout.
pub fn emit(v: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
    );
}
