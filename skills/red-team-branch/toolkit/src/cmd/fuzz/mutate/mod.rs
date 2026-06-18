mod confirm;
mod offer;
mod resume;

pub use offer::offer;

use super::{args::Args, report, session::Session};
use crate::config::Ctx;

pub const MAX_CORPUS: usize = 4096;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    let mut session = Session::start(args, ctx).await?;
    resume::resume(&mut session);
    session.run_loop().await;
    session.save_state();
    report::emit(&session);
    Ok(())
}
