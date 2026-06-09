use clap::{Parser, Subcommand};

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
    Serve,

    /// Run under Windows Service Control Manager.
    Service,
}
