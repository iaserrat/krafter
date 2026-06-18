const DEFAULT_PATH: &str = "/";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_PATH)]
    pub path: String,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
}
