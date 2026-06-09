use clap::{Args, Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(name = "winpassage-server")]
#[command(about = "WinPassage background agent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run as a foreground HTTP server.
    Serve(ServerOptions),

    /// Run under Windows Service Control Manager.
    Service(ServerOptions),
}

#[derive(Debug, Clone, Default, Args)]
pub struct ServerOptions {
    #[arg(long)]
    pub bind: Option<SocketAddr>,
    #[arg(long)]
    pub admin_token: Option<String>,
    #[arg(long, value_parser = clap::builder::BoolishValueParser::new())]
    pub require_tls: Option<bool>,
}
