mod audit;
mod cli;
mod config;
mod http;

#[cfg(windows)]
mod service;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "winpassage_server=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Serve => http::serve_until_shutdown(config::ServerConfig::from_env()?).await,
        Command::Service => run_as_service(),
    }
}

#[cfg(windows)]
fn run_as_service() -> Result<()> {
    service::run_service_dispatcher()
}

#[cfg(not(windows))]
fn run_as_service() -> Result<()> {
    anyhow::bail!("Windows Service mode is available only on Windows")
}
