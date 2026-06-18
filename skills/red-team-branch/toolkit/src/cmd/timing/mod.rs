//! `rtk timing` — measure response-latency differences between two inputs.
//! Interleaves variants to cancel drift and reports distributions.

mod args;
mod report;
mod signal;

#[cfg(test)]
mod tests;

use crate::config::Ctx;
use crate::http;

pub use args::Args;

pub async fn run(a: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let spec = super::base_spec(ctx, &a.method, &a.url, &a.headers, a.body)?;
    let client = http::build_client(&ctx.http)?;
    eprintln!("[rtk] timing: {} samples/variant, interleaved", a.samples);

    let spec_a = spec.render("VAR", &a.a_value);
    let spec_b = spec.render("VAR", &a.b_value);
    let (mut la, mut lb) = (Vec::new(), Vec::new());

    let _ = http::send_once(&client, &spec_a, http::NO_SNIPPET_LEN).await;
    for _ in 0..a.samples {
        sample(&client, &spec_a, &mut la).await;
        sample(&client, &spec_b, &mut lb).await;
    }

    let (Some(sa), Some(sb)) = (crate::util::stats(&la), crate::util::stats(&lb)) else {
        anyhow::bail!("no clean latency samples (target erroring? use --samples > 0)");
    };
    report::emit(&spec.url, &a.a_value, &a.b_value, sa, sb);
    Ok(())
}

// Record microsecond latency, dropping transport/error responses so failures
// can't inject 0us or noise into the distribution.
async fn sample(client: &reqwest::Client, spec: &http::RequestSpec, into: &mut Vec<u64>) {
    let out = http::send_once(client, spec, http::NO_SNIPPET_LEN).await;
    if out.error.is_some() {
        eprintln!("[rtk][WARN] timing: dropped errored sample: {:?}", out.error);
        return;
    }
    into.push(out.latency_us);
}
