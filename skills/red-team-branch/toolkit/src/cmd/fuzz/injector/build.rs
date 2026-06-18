use super::{
    channel::{authority_end, channel, Channel},
    model::{Injector, Safety},
};
use crate::{cmd::fuzz::args::Args, config::Ctx, http, util};

impl Injector {
    pub fn build(ctx: &Ctx, args: &Args) -> anyhow::Result<Self> {
        let url = http::resolve_url(ctx.base_url.as_deref(), &args.url)?;
        let headers = util::parse_headers(&args.headers);
        match channel(args, &url, &headers)? {
            Channel::Url => build_url(ctx, args, url, headers),
            Channel::Header => build_header(ctx, args, url, headers),
            Channel::Body => build_body(ctx, args, url, headers),
            Channel::Multipart => super::multipart::build(ctx, args, url, headers),
        }
    }
}

fn build_url(ctx: &Ctx, args: &Args, url: String, headers: Vec<(String, String)>) -> anyhow::Result<Injector> {
    let pos = url.find("{FUZZ}").ok_or_else(|| anyhow::anyhow!("--channel url but no {{FUZZ}} in URL"))?;
    if pos < authority_end(&url) {
        anyhow::bail!("{{FUZZ}} must be in the path/query/fragment, not the authority");
    }
    Ok(Injector::Url {
        method: args.method.clone(),
        prefix: url[..pos].to_string(),
        suffix: url[pos + "{FUZZ}".len()..].to_string(),
        headers,
        body: args.body.clone(),
        safety: Safety { allow_remote: ctx.allow_remote, allow_hosts: ctx.allow_hosts.clone() },
    })
}

fn build_header(ctx: &Ctx, args: &Args, url: String, mut headers: Vec<(String, String)>) -> anyhow::Result<Injector> {
    let idx = headers.iter().position(|(_, v)| v.contains("{FUZZ}"))
        .ok_or_else(|| anyhow::anyhow!("no {{FUZZ}} in any header value"))?;
    let (name, val) = headers.remove(idx);
    let (prefix, suffix) = val.split_once("{FUZZ}").unwrap();
    let safety = Safety { allow_remote: ctx.allow_remote, allow_hosts: ctx.allow_hosts.clone() };
    Ok(Injector::Header {
        method: args.method.clone(), url, headers, name,
        prefix: prefix.as_bytes().to_vec(), suffix: suffix.as_bytes().to_vec(),
        body: args.body.clone(), safety,
    })
}

fn build_body(ctx: &Ctx, args: &Args, url: String, headers: Vec<(String, String)>) -> anyhow::Result<Injector> {
    let body = args.body.clone().ok_or_else(|| anyhow::anyhow!("--channel body but no --body template"))?;
    let (prefix, suffix) = body.split_once("{FUZZ}").ok_or_else(|| anyhow::anyhow!("no {{FUZZ}} in body"))?;
    let safety = Safety { allow_remote: ctx.allow_remote, allow_hosts: ctx.allow_hosts.clone() };
    Ok(Injector::Body {
        method: args.method.clone(), url, headers,
        prefix: prefix.as_bytes().to_vec(), suffix: suffix.as_bytes().to_vec(), safety,
    })
}
