use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "WinPassage";
const DISPLAY_NAME: &str = "WinPassage Password Agent";

#[derive(Debug, Parser)]
#[command(name = "winpassage-agentctl")]
#[command(about = "Install and manage the WinPassage Windows Service")]
struct Cli {
    #[command(subcommand)]
    command: AgentCommand,
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    Install {
        #[arg(long)]
        server_bin: PathBuf,
        #[arg(long, default_value = "0.0.0.0:4487")]
        bind: SocketAddr,
        #[arg(long)]
        admin_token: String,
        #[arg(long, default_value = "false")]
        require_tls: String,
    },
    Uninstall,
    Start,
    Stop,
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        AgentCommand::Install {
            server_bin,
            bind,
            admin_token,
            require_tls,
        } => install(server_bin, bind, admin_token, require_tls),
        AgentCommand::Uninstall => sc(&["delete", SERVICE_NAME]),
        AgentCommand::Start => sc(&["start", SERVICE_NAME]),
        AgentCommand::Stop => sc(&["stop", SERVICE_NAME]),
        AgentCommand::Status => sc(&["query", SERVICE_NAME]),
    }
}

fn install(
    server_bin: PathBuf,
    bind: SocketAddr,
    admin_token: String,
    require_tls: String,
) -> Result<()> {
    if !cfg!(windows) {
        bail!("WinPassage service install is available only on Windows");
    }

    if admin_token.trim().is_empty() {
        bail!("--admin-token is required when installing the WinPassage service");
    }

    let require_tls = matches!(require_tls.as_str(), "1" | "true" | "TRUE" | "True");

    let server_bin = server_bin
        .canonicalize()
        .with_context(|| format!("server binary does not exist: {server_bin:?}"))?;

    let bin_path = format!(
        r#""{}" service --bind "{}" --admin-token "{}" --require-tls {}"#,
        server_bin.display(),
        bind,
        admin_token.trim(),
        require_tls
    );

    sc(&[
        "create",
        SERVICE_NAME,
        "binPath=",
        &bin_path,
        "start=",
        "auto",
        "DisplayName=",
        DISPLAY_NAME,
    ])?;

    sc(&[
        "description",
        SERVICE_NAME,
        "Local Windows password self-service agent",
    ])?;

    Ok(())
}

fn sc(args: &[&str]) -> Result<()> {
    if !cfg!(windows) {
        bail!("sc.exe is available only on Windows");
    }

    let status = Command::new("sc.exe")
        .args(args)
        .status()
        .context("failed to execute sc.exe")?;

    if !status.success() {
        bail!("sc.exe exited with status {status}");
    }

    Ok(())
}
