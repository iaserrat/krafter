use clap::Args as ClapArgs;

/// `ctk surface --paths a.rs b.ts` — the public contract surface of given files
/// (working-tree contents). The no-git building block behind `assess`.
#[derive(ClapArgs)]
pub struct Args {
    /// Files to read the contract from (working-tree contents).
    #[arg(long, required = true, num_args = 1..)]
    pub paths: Vec<String>,
}
