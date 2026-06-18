const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 400;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,
    #[arg(long)]
    pub ports: Option<String>,
    #[arg(long)]
    pub docker: bool,
    #[arg(long, default_value_t = DEFAULT_CONNECT_TIMEOUT_MS)]
    pub connect_timeout_ms: u64,
}
