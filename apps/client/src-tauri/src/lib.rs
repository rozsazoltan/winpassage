use winpassage_protocol::{ReconnectMappedDrivesRequest, ReconnectMappedDrivesResponse};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![reconnect_mapped_drives])
        .run(tauri::generate_context!())
        .expect("failed to run WinPassage Client");
}
