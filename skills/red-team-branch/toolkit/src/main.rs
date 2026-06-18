//! rtk — red-team toolkit. Offensive Swiss-army knife for the
//! `red-team-branch` skill. Local-only by default.
//!
//! Contract: stdout=one JSON doc, stderr=progress, local-only guard, no redirects.
mod cmd;
mod config;
mod http;
mod util;

use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rtk", version, about = "Offensive toolkit for the red-team-branch skill (local-only)")]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[arg(long, global = true)]
    allow_remote: bool,

    #[arg(long, global = true)]
    base_url: Option<String>,

    #[arg(long, global = true, value_name = "Header: value")]
    auth: Vec<String>,

    #[arg(long, global = true, value_name = "name: Header: value")]
    profile: Vec<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Recon(cmd::recon::Args),
    Callback(cmd::callback::Args),
    Sweep(cmd::sweep::Args),
    Bopla(cmd::bopla::Args),
    Matrix(cmd::matrix::Args),
    Discover(cmd::discover::Args),
    Params(cmd::params::Args),
    Fuzz(cmd::fuzz::Args),
    Race(cmd::race::Args),
    Timing(cmd::timing::Args),
    Sign(cmd::sign::Args),
    /// HTTP request smuggling: CL.TE, TE.CL, TE.TE via raw TCP.
    Smuggle(cmd::smuggle::Args),
    /// JWT attacks: alg=none, key confusion, blank secret, kid injection.
    Jwt(cmd::jwt::Args),
    /// CORS misconfig: origin reflection, null origin, wildcard+credentials.
    Cors(cmd::cors::Args),
    /// GraphQL probe: introspection, batching, alias bypass, GET queries.
    Gql(cmd::gql::Args),
    /// Response header audit: cookies, CSP, HSTS, XFO, Server disclosure.
    Headers(cmd::headers::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load(cli.config.as_deref())?;
    let mut ctx = cfg.into_ctx(cli.allow_remote);
    ctx.apply_overrides(cli.base_url, &cli.auth, &cli.profile)?;

    match cli.cmd {
        Cmd::Recon(a) => cmd::recon::run(a, &ctx).await,
        Cmd::Callback(a) => cmd::callback::run(a).await,
        Cmd::Sweep(a) => cmd::sweep::run(a, &ctx).await,
        Cmd::Bopla(a) => cmd::bopla::run(a, &ctx).await,
        Cmd::Matrix(a) => cmd::matrix::run(a, &ctx).await,
        Cmd::Discover(a) => cmd::discover::run(a, &ctx).await,
        Cmd::Params(a) => cmd::params::run(a, &ctx).await,
        Cmd::Fuzz(a) => cmd::fuzz::run(a, &ctx).await,
        Cmd::Race(a) => cmd::race::run(a, &ctx).await,
        Cmd::Timing(a) => cmd::timing::run(a, &ctx).await,
        Cmd::Sign(a) => cmd::sign::run(a),
        Cmd::Smuggle(a) => cmd::smuggle::run(a, &ctx).await,
        Cmd::Jwt(a) => cmd::jwt::run(a, &ctx).await,
        Cmd::Cors(a) => cmd::cors::run(a, &ctx).await,
        Cmd::Gql(a) => cmd::gql::run(a, &ctx).await,
        Cmd::Headers(a) => cmd::headers::run(a, &ctx).await,
    }
}
