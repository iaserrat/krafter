//! `rtk cors` — CORS misconfiguration probe. Tests target with multiple
//! Origin headers and flags reflected origins, wildcard credentials,
//! null-origin acceptance, and preflight bypasses.

mod args;
mod probe;
mod report;

#[cfg(test)]
mod tests;

pub use args::Args;

use crate::config::Ctx;
use crate::util;

const PREFLIGHT_METHOD: &str = "DELETE";

struct PreflightOutcome {
    result: probe::CorsResult,
    issues: Vec<String>,
}

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let extra = util::parse_headers(&args.headers);
    let origins = probe::build_origins(&args.custom_origin);
    let client = ctx.client_for(None)?;
    let mut results = Vec::new();
    let mut issues_per_origin = Vec::new();
    for origin in &origins {
        eprintln!("[rtk] cors: probing origin={origin}");
        let result = probe::probe(&client, ctx, &args.url, &args.method, origin, &extra).await?;
        let issues = report::classify(&result);
        issues_per_origin.push(issues);
        results.push(result);
    }
    if args.preflight {
        eprintln!("[rtk] cors: probing preflight");
        let preflight = probe_preflight(&client, ctx, &args.url, &extra).await;
        results.push(preflight.result);
        issues_per_origin.push(preflight.issues);
    }
    report::emit(&results, &issues_per_origin);
    Ok(())
}

async fn probe_preflight(client: &reqwest::Client, ctx: &Ctx, url: &str, extra: &[(String, String)]) -> PreflightOutcome {
    // probe() injects Origin itself; pass the preflight method header as extra.
    let mut pf_extra = vec![("Access-Control-Request-Method".to_string(), PREFLIGHT_METHOD.to_string())];
    pf_extra.extend_from_slice(extra);
    let result = probe::probe(client, ctx, url, "OPTIONS", probe::EVIL_ORIGIN, &pf_extra).await
        .unwrap_or_else(|_| probe::CorsResult { origin: "*preflight*".into(), acao: None, acac: None, acam: None, status: 0 });
    let mut issues = report::classify(&result);
    if report::preflight_accepted(&result, PREFLIGHT_METHOD) {
        issues.push("preflight-accepted".to_string());
    }
    PreflightOutcome { result, issues }
}