use super::boundary;
use super::model::{Injector, Safety};
use crate::cmd::fuzz::args::Args;
use crate::config::Ctx;

const BOUNDARY_SEED: u64 = 0;

pub fn build(ctx: &Ctx, args: &Args, url: String, headers: Vec<(String, String)>) -> anyhow::Result<Injector> {
    let body = args.body.as_ref().ok_or_else(|| anyhow::anyhow!("--channel multipart but no --body template"))?;
    let (prefix, suffix) = body.split_once("{FUZZ}").ok_or_else(|| anyhow::anyhow!("no {{FUZZ}} in --body template"))?;
    let boundary = boundary::random_boundary(&mut crate::cmd::mut_engine::Rng::seed(BOUNDARY_SEED));
    let field_name = args.multipart_field.clone();
    let filename = args.multipart_filename.clone();
    let preamble = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n");
    let postamble = format!("\r\n--{boundary}--\r\n");
    let safety = Safety { allow_remote: ctx.allow_remote, allow_hosts: ctx.allow_hosts.clone() };
    Ok(Injector::Multipart {
        method: args.method.clone(), url, headers, field_name, filename,
        body_prefix: prefix.as_bytes().to_vec(), body_suffix: suffix.as_bytes().to_vec(),
        preamble: preamble.into_bytes(), postamble: postamble.into_bytes(), boundary, safety,
    })
}
