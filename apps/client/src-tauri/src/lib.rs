use winpassage_protocol::{
    DriveOperationResponse, MountMappedDriveRequest, ReconnectMappedDrivesRequest,
    ReconnectMappedDrivesResponse,
};

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
    winpassage_windows::mount_mapped_drive(
        &request.username,
        &request.password,
        &request.drive,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn unmount_mapped_drive(letter: String) -> Result<DriveOperationResponse, String> {
    winpassage_windows::unmount_mapped_drive(&letter).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            reconnect_mapped_drives,
            mount_mapped_drive,
            unmount_mapped_drive
        ])
        .run(tauri::generate_context!())
        .expect("failed to run WinPassageClient");
}
