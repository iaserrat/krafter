const DEFAULT_CORS_METHOD: &str = "GET";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_CORS_METHOD)]
    pub method: String,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
    #[arg(long)]
    pub preflight: bool,
    #[arg(long = "origin", value_name = "ORIGIN")]
    pub custom_origin: Vec<String>,
}
