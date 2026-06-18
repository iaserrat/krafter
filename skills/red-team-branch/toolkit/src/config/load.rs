use super::{model, Config, Ctx};
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_NAMES: [&str; 2] = ["redteam.toml", "rtk.toml"];

impl Config {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Config> {
        match candidate(path) {
            Some(p) => read_config(&p),
            None => Ok(Config::default()),
        }
    }

    pub fn into_ctx(self, cli_allow_remote: bool) -> Ctx {
        let h = self.http;
        let concurrency = h.concurrency;
        let base_url = h.base_url.clone();
        Ctx {
            base_url,
            http: model::http_opts(h),
            concurrency,
            allow_remote: cli_allow_remote || self.safety.allow_remote,
            allow_hosts: self.safety.allow_hosts,
            profiles: self.profiles,
        }
    }
}

fn candidate(path: Option<&Path>) -> Option<PathBuf> {
    path.map(Path::to_path_buf).or_else(|| {
        DEFAULT_CONFIG_NAMES
            .iter()
            .map(PathBuf::from)
            .find(|p| p.exists())
    })
}

fn read_config(path: &Path) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("read config {}: {e}", path.display()))?;
    eprintln!("[rtk] using config: {}", path.display());
    Ok(toml::from_str(&text)?)
}
