mod args;
mod dictionary;
mod encoding;
mod injector;
mod minimize;
mod mutate;
mod payloads;
mod probe;
mod report;
mod session;
mod state;
mod static_mode;

pub use args::Args;

use crate::config::Ctx;

pub async fn run(args: Args, ctx: &Ctx) -> anyhow::Result<()> {
    if args.mutate {
        mutate::run(args, ctx).await
    } else {
        static_mode::run(args, ctx).await
    }
}
