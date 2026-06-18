const DEFAULT_METHODS: &str = "GET,POST,PUT,PATCH,DELETE";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_METHODS)]
    pub methods: String,
    #[arg(long)]
    pub profiles: Option<String>,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
    #[arg(long)]
    pub body: Option<String>,
}
