const DEFAULT_METHOD: &str = "POST";
const DEFAULT_CANARY: &str = "/smuggled-canary-rtk";
const SMUGGLE_TIMEOUT_MS: u64 = 3000;
const FOLLOWUP_TIMEOUT_MS: u64 = 5000;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_METHOD)]
    pub method: String,
    #[arg(long)]
    pub host_header: Option<String>,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
    #[arg(long, default_value = DEFAULT_CANARY)]
    pub canary_path: String,
    #[arg(long, default_value_t = SMUGGLE_TIMEOUT_MS)]
    pub smuggle_timeout_ms: u64,
    #[arg(long, default_value_t = FOLLOWUP_TIMEOUT_MS)]
    pub followup_timeout_ms: u64,
}
