use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const INSTALL_DIR: &str = r"C:\Program Files\WinPassage";
const SERVICE_NAME: &str = "WinPassage";
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
        "This Windows account is not a local administrator. Sign in with a local administrator account to use WinPassageAdmin."
            .to_string()
    } else if is_elevated {
        "Local administrator account detected. Server installation is available.".to_string()
    } else {
        "Local administrator account detected, but this process is not elevated. You can review settings, but installing or removing the service requires Run as administrator."
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
fn install_server_mode(request: InstallServerRequest) -> Result<ServerInstallResult, String> {
    ensure_elevated()?;
    validate_bind_host(&request.bind_host)?;
    if !request.admin_token.trim().is_empty() && request.admin_token.trim().len() < 16 {
        return Err(
            "Admin token must be at least 16 characters for a server installation.".to_string(),
        );
    }

    let install_dir = request
        .install_dir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(INSTALL_DIR));
    let source_dir = request
        .source_dir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_source_dir);

    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("failed to create install directory: {error}"))?;

    for binary in SERVICE_BINARIES {
        let source = source_dir.join(binary);
        let target = install_dir.join(binary);
        if !source.exists() {
            return Err(format!(
                "missing {binary} in {}. Place server, agentctl, and updater executables next to WinPassageAdmin or choose the binary source directory.",
                source_dir.display()
            ));
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

    let agentctl = install_dir.join("winpassage-agentctl.exe");
    let server_bin = install_dir.join("winpassage-server.exe");

    let _ = run_agentctl(&agentctl, &["stop"]);
    let _ = run_agentctl(&agentctl, &["uninstall"]);

    let bind = format!("{}:{}", request.bind_host, request.port);
    run_agentctl(
        &agentctl,
        &[
            "install",
            "--server-bin",
            &server_bin.display().to_string(),
            "--bind",
            &bind,
            "--admin-token",
            request.admin_token.trim(),
            "--require-tls",
            if request.require_tls { "true" } else { "false" },
        ],
    )?;
    run_agentctl(&agentctl, &["start"])?;

    Ok(ServerInstallResult {
        success: true,
        message: "WinPassage server mode installed and started on this computer.".to_string(),
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

    let install_dir = request
        .install_dir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(INSTALL_DIR));
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

fn ensure_elevated() -> Result<(), String> {
    if cfg!(windows) && !is_process_elevated() {
        return Err(
            "Elevation is required. Close WinPassageAdmin and reopen it with Run as administrator."
                .to_string(),
        );
    }

    Ok(())
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

fn default_source_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
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
            install_server_mode,
            demote_server_mode
        ])
        .run(tauri::generate_context!())
        .expect("failed to run WinPassageAdmin");
}
