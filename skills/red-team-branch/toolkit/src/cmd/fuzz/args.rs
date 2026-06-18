use std::path::PathBuf;

const DEFAULT_LEN_THRESHOLD: usize = 64;
const DEFAULT_MAX_EXEC: usize = 2000;
const DEFAULT_METHOD: &str = "GET";
const DEFAULT_CHANNEL: &str = "auto";
const DEFAULT_PLATEAU: usize = 8;

const DEFAULT_MULTIPART_FIELD: &str = "file";
const DEFAULT_MULTIPART_FILENAME: &str = "fuzz.bin";

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
    pub wordlist: Option<PathBuf>,
    #[arg(long)]
    pub payloads: Option<String>,
    #[arg(long)]
    pub match_substr: Option<String>,
    #[arg(long, default_value_t = DEFAULT_LEN_THRESHOLD)]
    pub len_threshold: usize,
    #[arg(long)]
    pub concurrency: Option<usize>,
    #[arg(long)]
    pub mutate: bool,
    #[arg(long)]
    pub seed: Option<u64>,
    #[arg(long, default_value_t = DEFAULT_MAX_EXEC)]
    pub max_exec: usize,
    #[arg(long, default_value = DEFAULT_CHANNEL)]
    pub channel: String,
    #[arg(long, default_value_t = DEFAULT_PLATEAU)]
    pub plateau: usize,
    #[arg(long)]
    pub max_time_secs: Option<u64>,
    #[arg(long)]
    pub state: Option<PathBuf>,
    #[arg(long, default_value = DEFAULT_MULTIPART_FIELD)]
    pub multipart_field: String,
    #[arg(long, default_value = DEFAULT_MULTIPART_FILENAME)]
    pub multipart_filename: String,
}
