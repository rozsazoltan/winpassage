use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use url::Url;

const OWNER: &str = "rozsazoltan";
const REPO: &str = "winpassage";
const GITHUB_HOST: &str = "github.com";
const GITHUB_API_HOST: &str = "api.github.com";

#[derive(Debug, Parser)]
#[command(name = "winpassage-updater")]
#[command(about = "Plan and verify WinPassage updates from the official GitHub repository")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the hardcoded update source used by WinPassage.
    Source,
    /// Print the release metadata endpoint for the requested channel.
    Plan {
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Check the official GitHub release metadata.
    Check,
    /// Boot-time boundary: check updates, then start the local service.
    Bootstrap {
        #[arg(long, default_value = r"C:\Program Files\WinPassage")]
        install_dir: String,
    },
    /// Verify whether a URL is accepted by the updater source policy.
    VerifyUrl { url: String },
    /// Print the expected asset name for a release component.
    Asset {
        #[arg(long)]
        version: String,
        #[arg(long)]
        component: Component,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Component {
    Server,
    Agentctl,
    Updater,
    Admin,
    Client,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Source => {
            println!("https://github.com/{OWNER}/{REPO}");
        }
        Command::Plan { version } => {
            let endpoint = release_metadata_url(&version);
            assert_allowed_update_url(&endpoint)?;
            println!("{endpoint}");
        }
        Command::Check => {
            let release = fetch_latest_release()?;
            println!("latest: {}", release.tag_name);
            if let Some(url) = release.html_url {
                println!("release: {url}");
            }
        }
        Command::Bootstrap { install_dir } => {
            let release = fetch_latest_release()?;
            println!("checked official release: {}", release.tag_name);
            start_local_service(PathBuf::from(install_dir))?;
        }
        Command::VerifyUrl { url } => {
            assert_allowed_update_url(&url)?;
            println!("accepted");
        }
        Command::Asset { version, component } => {
            println!("{}", expected_asset_name(&version, component));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: Option<String>,
}

fn fetch_latest_release() -> Result<GithubRelease> {
    let endpoint = release_metadata_url("latest");
    assert_allowed_update_url(&endpoint)?;

    reqwest::blocking::Client::builder()
        .user_agent("WinPassageUpdater")
        .build()?
        .get(endpoint)
        .send()?
        .error_for_status()?
        .json::<GithubRelease>()
        .context("failed to parse GitHub release metadata")
}

fn start_local_service(install_dir: PathBuf) -> Result<()> {
    let agentctl = install_dir.join("winpassage-agentctl.exe");
    if !agentctl.exists() {
        bail!(
            "winpassage-agentctl.exe was not found in {}",
            install_dir.display()
        );
    }

    let output = ProcessCommand::new(&agentctl).arg("start").output()?;
    if !output.status.success() {
        bail!(
            "failed to start WinPassage service: {}{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }

    println!("WinPassage service started");
    Ok(())
}

fn release_metadata_url(version: &str) -> String {
    if version == "latest" {
        format!("https://{GITHUB_API_HOST}/repos/{OWNER}/{REPO}/releases/latest")
    } else {
        let tag = if version.starts_with('v') {
            version.to_owned()
        } else {
            format!("v{version}")
        };
        format!("https://{GITHUB_API_HOST}/repos/{OWNER}/{REPO}/releases/tags/{tag}")
    }
}

fn expected_asset_name(version: &str, component: Component) -> String {
    let clean_version = version.strip_prefix('v').unwrap_or(version);

    match component {
        Component::Server => format!("winpassage-server-{clean_version}-windows-x64.exe"),
        Component::Agentctl => format!("winpassage-agentctl-{clean_version}-windows-x64.exe"),
        Component::Updater => format!("winpassage-updater-{clean_version}-windows-x64.exe"),
        Component::Admin => format!("WinPassageAdmin_{clean_version}_x64_en-US.msi"),
        Component::Client => format!("WinPassageClient_{clean_version}_x64_en-US.msi"),
    }
}

fn assert_allowed_update_url(raw: &str) -> Result<()> {
    let url = Url::parse(raw).with_context(|| format!("invalid update URL: {raw}"))?;

    if url.scheme() != "https" {
        bail!("update URL must use https");
    }

    let Some(host) = url.host_str() else {
        bail!("update URL must contain a host");
    };

    match host {
        GITHUB_HOST => assert_github_web_url(&url),
        GITHUB_API_HOST => assert_github_api_url(&url),
        _ => bail!("updates are allowed only from github.com/{OWNER}/{REPO}"),
    }
}

fn assert_github_web_url(url: &Url) -> Result<()> {
    let path = url.path();
    let required_prefix = format!("/{OWNER}/{REPO}/releases/download/");

    if !path.starts_with(&required_prefix) {
        bail!("GitHub download URL must point to {OWNER}/{REPO} release assets");
    }

    Ok(())
}

fn assert_github_api_url(url: &Url) -> Result<()> {
    let path = url.path();
    let releases_prefix = format!("/repos/{OWNER}/{REPO}/releases/");

    if !path.starts_with(&releases_prefix) {
        bail!("GitHub API URL must point to {OWNER}/{REPO} releases");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_official_release_asset_urls() {
        assert_allowed_update_url(
            "https://github.com/rozsazoltan/winpassage/releases/download/v0.1.0/winpassage-server-0.1.0-windows-x64.exe",
        )
        .expect("official release asset should be accepted");
    }

    #[test]
    fn accepts_official_release_metadata_urls() {
        assert_allowed_update_url(
            "https://api.github.com/repos/rozsazoltan/winpassage/releases/latest",
        )
        .expect("official release metadata should be accepted");
    }

    #[test]
    fn expected_asset_names_match_release_outputs() {
        assert_eq!(
            expected_asset_name("0.2.0", Component::Server),
            "winpassage-server-0.2.0-windows-x64.exe"
        );
        assert_eq!(
            expected_asset_name("v0.2.0", Component::Agentctl),
            "winpassage-agentctl-0.2.0-windows-x64.exe"
        );
        assert_eq!(
            expected_asset_name("0.2.0", Component::Admin),
            "WinPassageAdmin_0.2.0_x64_en-US.msi"
        );
        assert_eq!(
            expected_asset_name("v0.2.0", Component::Client),
            "WinPassageClient_0.2.0_x64_en-US.msi"
        );
    }

    #[test]
    fn rejects_non_https_urls() {
        let error = assert_allowed_update_url(
            "http://github.com/rozsazoltan/winpassage/releases/download/v0.1.0/file.exe",
        )
        .expect_err("http must be rejected");

        assert!(error.to_string().contains("https"));
    }

    #[test]
    fn rejects_other_repositories() {
        let error = assert_allowed_update_url(
            "https://github.com/attacker/winpassage/releases/download/v0.1.0/file.exe",
        )
        .expect_err("other repositories must be rejected");

        assert!(error.to_string().contains("rozsazoltan/winpassage"));
    }

    #[test]
    fn rejects_other_hosts() {
        let error = assert_allowed_update_url("https://example.com/winpassage/latest.exe")
            .expect_err("other hosts must be rejected");

        assert!(error.to_string().contains("github.com"));
    }
}
