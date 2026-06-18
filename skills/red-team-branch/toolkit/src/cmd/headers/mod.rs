//! `rtk headers` — response header security analysis. Fetches a URL,
//! inspects security headers (HSTS, CSP, XFO, X-CTO), cookie flags
//! (Secure, HttpOnly, SameSite), and Server disclosure.

mod args;
mod check;
mod cookie;
mod report;
#[cfg(test)]
mod tests;

pub use args::Args;

use crate::config::Ctx;
use crate::util;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let extra = util::parse_headers(&args.headers);
    let client = ctx.client_for(None)?;
    eprintln!("[rtk] headers: scanning {}{}", args.url, args.path);
    let report_data = check::fetch_headers(&client, ctx, &args.url, &args.path, &extra).await?;
    report::emit(&report_data);
    Ok(())
}
