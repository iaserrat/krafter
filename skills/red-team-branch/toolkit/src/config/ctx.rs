use crate::http::{self, HttpOpts};
use std::collections::BTreeMap;

pub struct Ctx {
    pub base_url: Option<String>,
    pub http: HttpOpts,
    pub concurrency: usize,
    pub allow_remote: bool,
    pub allow_hosts: Vec<String>,
    pub profiles: BTreeMap<String, BTreeMap<String, String>>,
}

impl Ctx {
    pub fn headers_for(&self, profile: Option<&str>) -> anyhow::Result<BTreeMap<String, String>> {
        match profile {
            None => Ok(self.http.headers.clone()),
            Some("anon") | Some("none") => Ok(BTreeMap::new()),
            Some(name) => self
                .profiles
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("unknown profile '{name}'")),
        }
    }

    pub fn client_for(&self, profile: Option<&str>) -> anyhow::Result<reqwest::Client> {
        let opts = HttpOpts {
            headers: self.headers_for(profile)?,
            ..self.http.clone()
        };
        http::build_client(&opts)
    }
}
