use super::Args;
use crate::{config::Ctx, http, util};
use serde_json::Value;

pub struct Target {
    pub write_url: String,
    pub read_url: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl Target {
    pub fn build(args: &Args, ctx: &Ctx) -> anyhow::Result<Self> {
        let write_url = guarded(ctx, &args.url)?;
        let read_url = args
            .read_url
            .as_ref()
            .map(|url| guarded(ctx, url))
            .transpose()?;
        Ok(Self {
            write_url,
            read_url,
            headers: util::parse_headers(&args.headers),
        })
    }
}

pub fn write(
    args: &Args,
    target: &Target,
    base: &serde_json::Map<String, Value>,
    key: &str,
    value: &Value,
) -> http::RequestSpec {
    let mut obj = base.clone();
    obj.insert(key.to_string(), value.clone());
    http::RequestSpec::new(&args.method, &target.write_url)
        .with_text_headers(&target.headers)
        .with_body(Some(Value::Object(obj).to_string()))
}

pub fn read(target: &Target) -> http::RequestSpec {
    http::RequestSpec::new("GET", target.read_url.as_ref().unwrap())
        .with_text_headers(&target.headers)
}

fn guarded(ctx: &Ctx, url: &str) -> anyhow::Result<String> {
    let resolved = http::resolve_url(ctx.base_url.as_deref(), url)?;
    http::guard_target(&resolved, ctx.allow_remote, &ctx.allow_hosts)?;
    Ok(resolved)
}
