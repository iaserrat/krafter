//! `rtk gql` — GraphQL security probe. Tests introspection exposure,
//! query batching, alias-based rate-limit bypass, and GET-based queries.

mod args;
mod probe;
mod report;
#[cfg(test)]
mod tests;

pub use args::Args;

use args::MAX_ALIAS_COUNT;
use crate::config::Ctx;
use crate::util;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let extra = util::parse_headers(&args.headers);
    let client = ctx.client_for(None)?;
    let url = format!("{}{}", args.url.trim_end_matches('/'), args.endpoint);
    let mut findings = report::GqlFindings {
        introspection_enabled: false, field_count: 0,
        batching_enabled: false, alias_accepted: false, alias_count: 0,
        get_query_enabled: false, issues: Vec::new(),
    };
    if args.introspection {
        let (enabled, fields) = probe::introspect(&client, ctx, &url, &extra).await?;
        findings.introspection_enabled = enabled;
        findings.field_count = fields;
    }
    if args.batching {
        findings.batching_enabled = probe::batch(&client, ctx, &url, &extra).await?;
    }
    if args.aliasing {
        let (accepted, count) = probe::alias_bypass(&client, ctx, &url, args.alias_count.min(MAX_ALIAS_COUNT), &extra).await?;
        findings.alias_accepted = accepted;
        findings.alias_count = count;
    }
    findings.get_query_enabled = probe::get_query(&client, ctx, &url, &extra).await?;
    report::emit(&findings);
    Ok(())
}
