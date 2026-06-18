const DEFAULT_SAMPLES: usize = 30;
const DEFAULT_METHOD: &str = "POST";

#[derive(clap::Args)]
pub struct Args {
    /// URL or path containing the `{VAR}` placeholder.
    #[arg(long)]
    pub(super) url: String,
    #[arg(long, default_value = DEFAULT_METHOD)]
    pub(super) method: String,
    #[arg(long = "header", value_name = "K: V")]
    pub(super) headers: Vec<String>,
    /// Body template containing `{VAR}`.
    #[arg(long)]
    pub(super) body: Option<String>,
    /// Value A (e.g. a valid username).
    #[arg(long)]
    pub(super) a_value: String,
    /// Value B (e.g. an invalid username).
    #[arg(long)]
    pub(super) b_value: String,
    /// Samples per variant.
    #[arg(long, default_value_t = DEFAULT_SAMPLES)]
    pub(super) samples: usize,
}
