mod args;
mod event;
mod log;
mod server;

pub use args::Args;

pub async fn run(args: Args) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || server::serve(args)).await?
}
