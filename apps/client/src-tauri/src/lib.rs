use serde::{Deserialize, Serialize};
use url::Url;
use winpassage_protocol::{
    DriveOperationResponse, MountMappedDriveRequest, ReconnectMappedDrivesRequest,
    ReconnectMappedDrivesResponse,
};

const OWNER: &str = "rozsazoltan";
const REPO: &str = "winpassage";
const GITHUB_API_HOST: &str = "api.github.com";

#[derive(Debug, Serialize)]
struct UpdateStatus {
    current_version: String,
    latest_version: Option<String>,
    update_available: bool,
    release_url: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: Option<String>,
}

#[tauri::command]
fn reconnect_mapped_drives(
    request: ReconnectMappedDrivesRequest,
) -> Result<ReconnectMappedDrivesResponse, String> {
    let results = winpassage_windows::reconnect_mapped_drives(
        &request.username,
        &request.password,
        &request.drives,
    )
    .map_err(|error| error.to_string())?;

    Ok(ReconnectMappedDrivesResponse { results })
}

#[tauri::command]
fn mount_mapped_drive(request: MountMappedDriveRequest) -> Result<DriveOperationResponse, String> {
    winpassage_windows::mount_mapped_drive(&request.username, &request.password, &request.drive)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn unmount_mapped_drive(letter: String) -> Result<DriveOperationResponse, String> {
    winpassage_windows::unmount_mapped_drive(&letter).map_err(|error| error.to_string())
}

#[tauri::command]
fn check_for_updates() -> Result<UpdateStatus, String> {
    let url = format!("https://{GITHUB_API_HOST}/repos/{OWNER}/{REPO}/releases/latest");
    assert_allowed_api_url(&url)?;

    let release = reqwest::blocking::Client::builder()
        .user_agent("WinPassageClient")
        .build()
        .map_err(|error| format!("failed to create update client: {error}"))?
        .get(url)
        .send()
        .map_err(|error| format!("failed to query GitHub release metadata: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub release metadata request failed: {error}"))?
        .json::<GithubRelease>()
        .map_err(|error| format!("failed to parse GitHub release metadata: {error}"))?;

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

fn assert_allowed_api_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|error| format!("invalid update URL: {error}"))?;
    if url.scheme() != "https" {
        return Err("update URL must use https".to_string());
    }
    if url.host_str() != Some(GITHUB_API_HOST) {
        return Err(format!(
            "updates are allowed only from github.com/{OWNER}/{REPO}"
        ));
    }
    let prefix = format!("/repos/{OWNER}/{REPO}/releases/");
    if !url.path().starts_with(&prefix) {
        return Err(format!(
            "GitHub API URL must point to {OWNER}/{REPO} releases"
        ));
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            reconnect_mapped_drives,
            mount_mapped_drive,
            unmount_mapped_drive,
            check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run WinPassageClient");
}
