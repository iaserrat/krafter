const DEFAULT_ENDPOINT: &str = "/graphql";
const DEFAULT_ALIAS_COUNT: usize = 50;
pub const MAX_ALIAS_COUNT: usize = 500;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub url: String,
    #[arg(long, default_value = DEFAULT_ENDPOINT)]
    pub endpoint: String,
    #[arg(long)]
    pub introspection: bool,
    #[arg(long)]
    pub batching: bool,
    #[arg(long)]
    pub aliasing: bool,
    #[arg(long, default_value_t = DEFAULT_ALIAS_COUNT)]
    pub alias_count: usize,
    #[arg(long = "header", value_name = "K: V")]
    pub headers: Vec<String>,
}
