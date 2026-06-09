mod audit;
mod cli;
mod config;
mod http;

#[cfg(windows)]
mod service;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, ServerOptions};

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
        Command::Serve(options) => {
            http::serve_until_shutdown(config_from_options(options)?).await
        }
        Command::Service(options) => run_as_service(options),
    }
}

fn config_from_options(options: ServerOptions) -> Result<config::ServerConfig> {
    config::ServerConfig::from_env_with_overrides(
        options.bind,
        options.admin_token,
        options.require_tls,
    )
}

#[cfg(windows)]
fn run_as_service(options: ServerOptions) -> Result<()> {
    service::run_service_dispatcher(options)
}

#[cfg(not(windows))]
fn run_as_service(_options: ServerOptions) -> Result<()> {
    anyhow::bail!("Windows Service mode is available only on Windows")
}
