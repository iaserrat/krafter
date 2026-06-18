use std::path::PathBuf;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub base: Option<String>,
    #[arg(long)]
    pub wordlist: Option<PathBuf>,
    #[arg(long)]
    pub concurrency: Option<usize>,
}
