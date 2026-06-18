use clap::Args as ClapArgs;

/// MOSS noise threshold: the k-gram length. Below it, matches are coincidental.
const DUP_K: usize = 12;
/// Shortest reported clone, in normalized tokens.
const DUP_MIN_TOKENS: usize = 50;
/// Secondary floor: kills one-line accidental matches.
const DUP_MIN_LINES: usize = 5;

/// `cqt dup` — deterministic clone detection. Default scope "branch" reports
/// only clones the branch introduces (incl. copies of existing repo code);
/// "repo" is a whole-repo census.
#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = "branch")]
    pub scope: String,
    /// Exact clones only (no identifier/literal normalization).
    #[arg(long)]
    pub type1: bool,
    #[arg(long, default_value_t = DUP_MIN_TOKENS)]
    pub min_tokens: usize,
    #[arg(long, default_value_t = DUP_K)]
    pub k: usize,
    #[arg(long, default_value_t = DUP_MIN_LINES)]
    pub min_lines: usize,
}
