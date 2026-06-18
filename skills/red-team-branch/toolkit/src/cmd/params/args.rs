use std::path::PathBuf;

const DEFAULT_METHOD: &str = "GET";
const DEFAULT_LOCATION: &str = "query";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_METHOD)]
    pub method: String,
    #[arg(long, default_value = DEFAULT_LOCATION, value_parser = ["query", "json", "form"])]
    pub location: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
    #[arg(long)]
    pub wordlist: Option<PathBuf>,
    #[arg(long = "as", value_name = "PROFILE")]
    pub as_profile: Option<String>,
    #[arg(long)]
    pub concurrency: Option<usize>,
}
