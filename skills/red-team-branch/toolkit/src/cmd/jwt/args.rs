const DEFAULT_VERIFY_URL: &str = "/";
const DEFAULT_ALGORITHM: &str = "HS256";

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub token: String,
    #[arg(long, default_value = DEFAULT_VERIFY_URL)]
    pub verify_url: Option<String>,
    #[arg(long)]
    pub header_name: Option<String>,
    #[arg(long)]
    pub public_key: Option<String>,
    #[arg(long)]
    pub kid_path: Option<String>,
    #[arg(long, default_value = DEFAULT_ALGORITHM)]
    pub algorithm: String,
}
