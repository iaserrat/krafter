const DEFAULT_REQUEST_COUNT: usize = 20;
const DEFAULT_METHOD: &str = "POST";
const DEFAULT_SUCCESS_RANGE: &str = "200-299";

#[derive(clap::Args)]
pub struct Args {
    /// URL or path to hammer.
    #[arg(long)]
    pub(super) url: String,
    #[arg(long, default_value = DEFAULT_METHOD)]
    pub(super) method: String,
    #[arg(long = "header", value_name = "K: V")]
    pub(super) headers: Vec<String>,
    #[arg(long)]
    pub(super) body: Option<String>,
    /// Number of simultaneous requests.
    #[arg(long, default_value_t = DEFAULT_REQUEST_COUNT)]
    pub(super) count: usize,
    /// Status range counted as "success", e.g. "200-299".
    #[arg(long, default_value = DEFAULT_SUCCESS_RANGE)]
    pub(super) success: String,
}
