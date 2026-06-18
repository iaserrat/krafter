use std::path::PathBuf;

const DEFAULT_METHOD: &str = "GET";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_METHOD)]
    pub method: String,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub compare: Option<String>,
    #[arg(long)]
    pub range: Option<String>,
    #[arg(long)]
    pub ids: Option<String>,
    #[arg(long)]
    pub ids_file: Option<PathBuf>,
    #[arg(long)]
    pub concurrency: Option<usize>,
}
