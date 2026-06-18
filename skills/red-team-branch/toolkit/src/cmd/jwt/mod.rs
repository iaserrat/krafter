//! `rtk jwt` — JWT attack probe. Parses a JWT, generates forged tokens
//! (alg=none, key confusion, blank secret, kid injection), and verifies
//! acceptance against a target endpoint.

mod args;
mod attack;
mod crypto;
mod parse;
mod report;

pub use args::Args;

use crate::{cmd, config::Ctx, http};

const AUTH_HEADER_PREFIX: &str = "Bearer ";
const DEFAULT_HEADER_NAME: &str = "Authorization";
const DEFAULT_VERIFY_URL: &str = "/";

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let jwt = parse::parse(&args.token)?;
    let attacks = attack::all_attacks(
        &jwt,
        &args.algorithm,
        args.public_key.as_deref(),
        args.kid_path.as_deref(),
    );
    let client = ctx.client_for(None)?;
    let verify_url = args.verify_url.as_deref().unwrap_or(DEFAULT_VERIFY_URL);
    let header_name = args.header_name.as_deref().unwrap_or(DEFAULT_HEADER_NAME);
    let spec = cmd::base_spec(ctx, "GET", verify_url, &[], None)?;
    // Control: an invalid-signature token. If the endpoint accepts it too, it
    // is not validating signatures and no forgery below proves a bypass.
    let control = attack::control_token(&jwt);
    let control_status = try_token(&client, &spec, header_name, &control).await.status;
    eprintln!("[rtk] jwt: control (invalid sig) -> {control_status}");
    let mut findings = Vec::new();
    for atk in &attacks {
        eprintln!("[rtk] jwt: testing {}", atk.name);
        let outcome = try_token(&client, &spec, header_name, &atk.token).await;
        findings.push(report::classify(atk, outcome.status, control_status));
    }
    report::emit(&findings, control_status);
    Ok(())
}

async fn try_token(client: &reqwest::Client, base: &http::RequestSpec, header_name: &str, token: &str) -> http::Outcome {
    let value = if header_name.eq_ignore_ascii_case("Authorization") {
        format!("{AUTH_HEADER_PREFIX}{token}")
    } else {
        token.to_string()
    };
    let mut spec = base.clone();
    spec.headers = vec![http::RequestHeader::Text(header_name.to_string(), value)];
    http::send_once(client, &spec, http::NO_SNIPPET_LEN).await
}

#[cfg(test)]
mod tests;
