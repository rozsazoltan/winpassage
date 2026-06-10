use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

const INSTALL_DIR: &str = r"C:\Program Files\WinPassage";
const SERVICE_NAME: &str = "WinPassage";
const OWNER: &str = "rozsazoltan";
const REPO: &str = "winpassage";
const GITHUB_HOST: &str = "github.com";
const GITHUB_API_HOST: &str = "api.github.com";
const SERVICE_BINARIES: [&str; 3] = [
    "winpassage-server.exe",
    "winpassage-agentctl.exe",
    "winpassage-updater.exe",
];

#[derive(Debug, Serialize)]
struct AdminHostStatus {
    is_windows: bool,
    is_admin_account: bool,
    is_elevated: bool,
    service_installed: bool,
    install_dir: String,
    executable_dir: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct UpdateStatus {
    current_version: String,
    latest_version: Option<String>,
    update_available: bool,
    release_url: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct InstallServerRequest {
    #[serde(default)]
    source_dir: Option<String>,
    #[serde(default)]
    install_dir: Option<String>,
    bind_host: String,
    port: u16,
    admin_token: String,
    require_tls: bool,
}

#[derive(Debug, Deserialize)]
struct DemoteServerRequest {
    #[serde(default)]
    install_dir: Option<String>,
    confirmation: String,
    remove_files: bool,
}

#[derive(Debug, Serialize)]
struct ServerInstallResult {
    success: bool,
    message: String,
    install_dir: String,
    server_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[tauri::command]
fn get_admin_host_status() -> AdminHostStatus {
    let is_windows = cfg!(windows);
    let is_admin_account = is_local_admin_account();
    let is_elevated = is_process_elevated();
    let service_installed = is_service_installed();
    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.display().to_string());

    let message = if !is_windows {
        "WinPassage server installation is available only on Windows.".to_string()
    } else if !is_admin_account {
        "Sign in with a local administrator account to use WinPassageAdmin.".to_string()
    } else if is_elevated {
        "Administrator privileges are available.".to_string()
    } else {
        "You are using an administrator account. Service install and removal require Run as administrator."
            .to_string()
    };

    AdminHostStatus {
        is_windows,
        is_admin_account,
        is_elevated,
        service_installed,
        install_dir: INSTALL_DIR.to_string(),
        executable_dir,
        message,
    }
}

#[tauri::command]
fn check_for_updates() -> Result<UpdateStatus, String> {
    let release = fetch_release("latest")?;
    let latest = release.tag_name.trim_start_matches('v').to_string();
    let current = env!("CARGO_PKG_VERSION").to_string();
    let available = latest != current;

    Ok(UpdateStatus {
        current_version: current.clone(),
        latest_version: Some(latest.clone()),
        update_available: available,
        release_url: release.html_url,
        message: if available {
            format!("WinPassage {latest} is available.")
        } else {
            format!("WinPassage {current} is current.")
        },
    })
}

#[tauri::command]
fn prepare_server_binaries(install_dir: Option<String>) -> Result<ServerInstallResult, String> {
    ensure_elevated()?;
    let install_dir = resolve_install_dir(install_dir.as_deref());
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("failed to create install directory: {error}"))?;
    download_service_binaries(&install_dir)?;

    Ok(ServerInstallResult {
        success: true,
        message: "Downloaded and verified WinPassage server components from GitHub.".to_string(),
        install_dir: install_dir.display().to_string(),
        server_url: String::new(),
    })
}

#[tauri::command]
fn install_server_mode(request: InstallServerRequest) -> Result<ServerInstallResult, String> {
    ensure_elevated()?;
    validate_bind_host(&request.bind_host)?;
    if !request.admin_token.trim().is_empty() && request.admin_token.trim().len() < 16 {
        return Err(
            "Admin token must be at least 16 characters, or leave it empty to generate one."
                .to_string(),
        );
    }

    let install_dir = resolve_install_dir(request.install_dir.as_deref());
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("failed to create install directory: {error}"))?;

    let existing_agentctl = install_dir.join("winpassage-agentctl.exe");
    if existing_agentctl.exists() {
        let _ = run_agentctl(&existing_agentctl, &["stop"]);
        let _ = run_agentctl(&existing_agentctl, &["uninstall"]);
    }

    if let Some(source_dir) = request
        .source_dir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
    {
        copy_service_binaries(&source_dir, &install_dir)?;
    } else {
        download_service_binaries(&install_dir)?;
    }

    let agentctl = install_dir.join("winpassage-agentctl.exe");
    let server_bin = install_dir.join("winpassage-server.exe");

    let bind = format!("{}:{}", request.bind_host, request.port);
    let token = if request.admin_token.trim().is_empty() {
        generate_admin_token()
    } else {
        request.admin_token.trim().to_string()
    };

    run_agentctl(
        &agentctl,
        &[
            "install",
            "--server-bin",
            &server_bin.display().to_string(),
            "--bind",
            &bind,
            "--admin-token",
            &token,
            "--require-tls",
            if request.require_tls { "true" } else { "false" },
        ],
    )?;
    run_agentctl(&agentctl, &["start"])?;

    Ok(ServerInstallResult {
        success: true,
        message: "WinPassage server installed from verified GitHub release assets.".to_string(),
        install_dir: install_dir.display().to_string(),
        server_url: format!("http://{}:{}", request.bind_host, request.port),
    })
}

#[tauri::command]
fn demote_server_mode(request: DemoteServerRequest) -> Result<ServerInstallResult, String> {
    ensure_elevated()?;
    if request.confirmation.trim() != "REMOVE SERVER" {
        return Err("Type REMOVE SERVER to confirm server removal.".to_string());
    }

    let install_dir = resolve_install_dir(request.install_dir.as_deref());
    let agentctl = install_dir.join("winpassage-agentctl.exe");

    if agentctl.exists() {
        let _ = run_agentctl(&agentctl, &["stop"]);
        run_agentctl(&agentctl, &["uninstall"])?;
    }

    if request.remove_files {
        for binary in SERVICE_BINARIES {
            let path = install_dir.join(binary);
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
            }
        }
    }

    Ok(ServerInstallResult {
        success: true,
        message: "WinPassage server service removed from this computer.".to_string(),
        install_dir: install_dir.display().to_string(),
        server_url: String::new(),
    })
}

fn resolve_install_dir(value: Option<&str>) -> PathBuf {
    value
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(INSTALL_DIR))
}

fn ensure_elevated() -> Result<(), String> {
    if cfg!(windows) && !is_process_elevated() {
        return Err(
            "Run WinPassageAdmin as administrator to install or remove the local server."
                .to_string(),
        );
    }

    Ok(())
}

fn copy_service_binaries(source_dir: &Path, install_dir: &Path) -> Result<(), String> {
    for binary in SERVICE_BINARIES {
        let source = source_dir.join(binary);
        let target = install_dir.join(binary);
        if !source.exists() {
            return Err(format!("missing {binary} in {}", source_dir.display()));
        }
        if source != target {
            fs::copy(&source, &target).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {error}",
                    source.display(),
                    target.display()
                )
            })?;
        }
    }

    Ok(())
}

fn download_service_binaries(install_dir: &Path) -> Result<(), String> {
    let release = fetch_release("latest")?;
    let version = release.tag_name.trim_start_matches('v');
    let components = [
        (
            "server",
            format!("winpassage-server-{version}-windows-x64.exe"),
        ),
        (
            "agentctl",
            format!("winpassage-agentctl-{version}-windows-x64.exe"),
        ),
        (
            "updater",
            format!("winpassage-updater-{version}-windows-x64.exe"),
        ),
    ];

    for (component, asset_name) in components {
        let asset = find_asset(&release, &asset_name)?;
        let checksum_asset = find_asset(&release, &format!("{asset_name}.sha256"))?;
        let bytes = download_asset(&asset.browser_download_url)?;
        let checksum_text =
            String::from_utf8(download_asset(&checksum_asset.browser_download_url)?)
                .map_err(|error| format!("invalid checksum file for {asset_name}: {error}"))?;
        verify_sha256(&bytes, &checksum_text, &asset_name)?;

        let target_name = format!("winpassage-{component}.exe");
        let target = install_dir.join(target_name);
        let tmp = target.with_extension("exe.download");
        let mut file = fs::File::create(&tmp)
            .map_err(|error| format!("failed to create {}: {error}", tmp.display()))?;
        file.write_all(&bytes)
            .map_err(|error| format!("failed to write {}: {error}", tmp.display()))?;
        fs::rename(&tmp, &target).map_err(|error| {
            format!(
                "failed to move {} to {}: {error}",
                tmp.display(),
                target.display()
            )
        })?;
    }

    Ok(())
}

fn fetch_release(version: &str) -> Result<GithubRelease, String> {
    let url = if version == "latest" {
        format!("https://{GITHUB_API_HOST}/repos/{OWNER}/{REPO}/releases/latest")
    } else {
        let tag = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{version}")
        };
        format!("https://{GITHUB_API_HOST}/repos/{OWNER}/{REPO}/releases/tags/{tag}")
    };
    assert_allowed_update_url(&url)?;

    reqwest::blocking::Client::builder()
        .user_agent("WinPassageAdmin")
        .build()
        .map_err(|error| format!("failed to create update client: {error}"))?
        .get(url)
        .send()
        .map_err(|error| format!("failed to query GitHub release metadata: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub release metadata request failed: {error}"))?
        .json::<GithubRelease>()
        .map_err(|error| format!("failed to parse GitHub release metadata: {error}"))
}

fn find_asset<'a>(release: &'a GithubRelease, name: &str) -> Result<&'a GithubAsset, String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| format!("official release asset not found: {name}"))
}

fn download_asset(url: &str) -> Result<Vec<u8>, String> {
    assert_allowed_update_url(url)?;
    reqwest::blocking::Client::builder()
        .user_agent("WinPassageAdmin")
        .build()
        .map_err(|error| format!("failed to create update client: {error}"))?
        .get(url)
        .send()
        .map_err(|error| format!("failed to download release asset: {error}"))?
        .error_for_status()
        .map_err(|error| format!("release asset download failed: {error}"))?
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| format!("failed to read release asset: {error}"))
}

fn verify_sha256(bytes: &[u8], checksum_text: &str, asset_name: &str) -> Result<(), String> {
    let expected = checksum_text
        .split_whitespace()
        .find(|part| part.len() == 64 && part.chars().all(|ch| ch.is_ascii_hexdigit()))
        .ok_or_else(|| format!("missing sha256 checksum for {asset_name}"))?
        .to_ascii_lowercase();
    let actual = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    if actual != expected {
        return Err(format!(
            "sha256 mismatch for {asset_name}; expected {expected}, got {actual}"
        ));
    }

    Ok(())
}

fn assert_allowed_update_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|error| format!("invalid update URL: {error}"))?;
    if url.scheme() != "https" {
        return Err("update URL must use https".to_string());
    }
    match url.host_str() {
        Some(GITHUB_HOST) => {
            let prefix = format!("/{OWNER}/{REPO}/releases/download/");
            if !url.path().starts_with(&prefix) {
                return Err(format!(
                    "GitHub download URL must point to {OWNER}/{REPO} release assets"
                ));
            }
        }
        Some(GITHUB_API_HOST) => {
            let prefix = format!("/repos/{OWNER}/{REPO}/releases/");
            if !url.path().starts_with(&prefix) {
                return Err(format!(
                    "GitHub API URL must point to {OWNER}/{REPO} releases"
                ));
            }
        }
        _ => {
            return Err(format!(
                "updates are allowed only from github.com/{OWNER}/{REPO}"
            ))
        }
    }

    Ok(())
}

fn generate_admin_token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("wp-{nanos:x}-local-admin-token")
}

fn is_local_admin_account() -> bool {
    #[cfg(windows)]
    {
        Command::new("whoami")
            .args(["/groups"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                output.status.success() && stdout.contains("S-1-5-32-544")
            })
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        true
    }
}

fn is_process_elevated() -> bool {
    #[cfg(windows)]
    {
        Command::new("net")
            .arg("session")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        true
    }
}

fn is_service_installed() -> bool {
    #[cfg(windows)]
    {
        Command::new("sc")
            .args(["query", SERVICE_NAME])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        false
    }
}

fn validate_bind_host(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Bind host is required.".to_string());
    }

    if trimmed.contains('/') || trimmed.contains(':') {
        return Err(
            "Bind host must be an IP address or hostname without protocol, path, or port."
                .to_string(),
        );
    }

    Ok(())
}

fn run_agentctl(agentctl: &Path, args: &[&str]) -> Result<(), String> {
    let output = Command::new(agentctl)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run {}: {error}", agentctl.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{} {} failed: {}{}",
            agentctl.display(),
            args.join(" "),
            stderr,
            stdout
        ));
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_admin_host_status,
            check_for_updates,
            prepare_server_binaries,
            install_server_mode,
            demote_server_mode,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run WinPassageAdmin");
}
