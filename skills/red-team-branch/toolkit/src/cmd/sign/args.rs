use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub secret: String,
    #[arg(long)]
    pub payload: Option<String>,
    #[arg(long)]
    pub payload_file: Option<PathBuf>,
    #[arg(long, default_value = "sha256", value_parser = ["sha256", "sha1"])]
    pub algo: String,
    #[arg(long, default_value = "hex", value_parser = ["hex", "base64"])]
    pub encoding: String,
    #[arg(long, default_value = "raw", value_parser = ["raw", "stripe"])]
    pub format: String,
    #[arg(long)]
    pub timestamp: Option<i64>,
}
