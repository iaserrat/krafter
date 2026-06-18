#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = "PATCH")]
    pub method: String,
    #[arg(long)]
    pub read_url: Option<String>,
    #[arg(long, default_value = "{}")]
    pub body: String,
    #[arg(long = "field", value_name = "K=V")]
    pub fields: Vec<String>,
    #[arg(long = "as", value_name = "PROFILE")]
    pub as_profile: Option<String>,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
}
