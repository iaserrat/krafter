use std::path::PathBuf;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9099;
const DEFAULT_RESPONSE_BODY: &str = "rtk-callback ok";
const DEFAULT_RESPONSE_STATUS: u16 = 200;
const DEFAULT_MAX_BODY_BYTES: u64 = 8192;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    #[arg(long)]
    pub log_file: Option<PathBuf>,
    #[arg(long, default_value = DEFAULT_RESPONSE_BODY)]
    pub respond_body: String,
    #[arg(long, default_value_t = DEFAULT_RESPONSE_STATUS)]
    pub respond_status: u16,
    #[arg(long, default_value_t = DEFAULT_MAX_BODY_BYTES)]
    pub max_body: u64,
}
