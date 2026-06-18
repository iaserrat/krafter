use crate::{cmd, config::Ctx, http};

const NULL_ORIGIN: &str = "null";
pub const EVIL_ORIGIN: &str = "https://evil.com";
const SUBDOMAIN_ORIGIN: &str = "https://evil.target.com";

pub struct CorsResult {
    pub origin: String,
    pub acao: Option<String>,
    pub acac: Option<String>,
    pub acam: Option<String>,
    pub status: u16,
}

pub fn build_origins(custom: &[String]) -> Vec<String> {
    if custom.is_empty() {
        vec![EVIL_ORIGIN.to_string(), NULL_ORIGIN.to_string(), SUBDOMAIN_ORIGIN.to_string()]
    } else {
        custom.to_vec()
    }
}

pub async fn probe(client: &reqwest::Client, ctx: &Ctx, url: &str, method: &str, origin: &str, extra: &[(String, String)]) -> anyhow::Result<CorsResult> {
    let mut headers: Vec<String> = vec![format!("Origin: {origin}")];
    for (k, v) in extra {
        headers.push(format!("{k}: {v}"));
    }
    let spec = cmd::base_spec(ctx, method, url, &headers, None)?;
    let out = http::send_once(client, &spec, http::NO_SNIPPET_LEN).await;
    let acao = http::find_header(&out, "access-control-allow-origin");
    let acac = http::find_header(&out, "access-control-allow-credentials");
    let acam = http::find_header(&out, "access-control-allow-methods");
    Ok(CorsResult { origin: origin.to_string(), acao, acac, acam, status: out.status })
}
