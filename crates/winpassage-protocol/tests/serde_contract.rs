use uuid::Uuid;
use winpassage_protocol::{
    ActionResponse, ChangeOwnPasswordRequest, CreateLocalUserRequest, DeleteLocalUserRequest,
    DriveMapping, ReconnectMappedDrivesRequest, ServerEndpointProfile,
};

#[test]
fn server_endpoint_profile_round_trips_without_secret_fields() {
    let profile = ServerEndpointProfile {
        id: "office-main".to_string(),
        name: "Office Main".to_string(),
        network_name: "Budapest office".to_string(),
        protocol: "https".to_string(),
        host: "192.168.1.10".to_string(),
        port: 4487,
        notes: Some("Primary small office server".to_string()),
    };

    let json = serde_json::to_string(&profile).expect("profile should serialize");
    assert!(json.contains("office-main"));
    assert!(!json.to_ascii_lowercase().contains("token"));
    assert!(!json.to_ascii_lowercase().contains("password"));

    let decoded: ServerEndpointProfile =
        serde_json::from_str(&json).expect("profile should decode");
    assert_eq!(decoded.host, "192.168.1.10");
    assert_eq!(decoded.port, 4487);
}

#[test]
fn create_local_user_request_keeps_reason_and_request_id() {
    let request_id = Uuid::nil();
    let payload = CreateLocalUserRequest {
        username: "julia".to_string(),
        full_name: Some("Julia Nagy".to_string()),
        password: "InitialPassword123!".to_string(),
        must_change_password: true,
        enabled: true,
        admin: false,
        reason: Some("new employee".to_string()),
        request_id: Some(request_id),
    };

    let json = serde_json::to_string(&payload).expect("request should serialize");
    let decoded: CreateLocalUserRequest =
        serde_json::from_str(&json).expect("request should decode");

    assert_eq!(decoded.username, "julia");
    assert_eq!(decoded.request_id, Some(request_id));
    assert_eq!(decoded.reason.as_deref(), Some("new employee"));
}

#[test]
fn delete_request_requires_explicit_profile_and_session_choices() {
    let json = r#"{
        "delete_profile": true,
        "logoff_sessions": true,
        "reason": "offboarding",
        "request_id": null
    }"#;

    let decoded: DeleteLocalUserRequest =
        serde_json::from_str(json).expect("delete request should decode");
    assert!(decoded.delete_profile);
    assert!(decoded.logoff_sessions);
    assert_eq!(decoded.reason.as_deref(), Some("offboarding"));
}

#[test]
fn self_service_password_change_contract_is_user_scoped() {
    let json = r#"{
        "username": "julia",
        "current_password": "OldPassword123!",
        "new_password": "NewPassword123!",
        "request_id": null
    }"#;

    let decoded: ChangeOwnPasswordRequest =
        serde_json::from_str(json).expect("change request should decode");
    assert_eq!(decoded.username, "julia");
    assert_eq!(decoded.current_password, "OldPassword123!");
    assert_eq!(decoded.new_password, "NewPassword123!");
}

#[test]
fn mapped_drive_reconnect_contract_preserves_per_drive_targets() {
    let request = ReconnectMappedDrivesRequest {
        username: "julia".to_string(),
        password: "NewPassword123!".to_string(),
        drives: vec![
            DriveMapping {
                letter: "S:".to_string(),
                remote_path: r"\\server\shared".to_string(),
            },
            DriveMapping {
                letter: "I:".to_string(),
                remote_path: r"\\server\internal".to_string(),
            },
        ],
    };

    let json = serde_json::to_string(&request).expect("drive request should serialize");
    let decoded: ReconnectMappedDrivesRequest =
        serde_json::from_str(&json).expect("drive request should decode");

    assert_eq!(decoded.drives.len(), 2);
    assert_eq!(decoded.drives[0].letter, "S:");
    assert_eq!(decoded.drives[1].remote_path, r"\\server\internal");
}

#[test]
fn action_response_round_trips_request_id() {
    let request_id = Uuid::nil();
    let response = ActionResponse {
        success: true,
        message: "ok".to_string(),
        request_id,
    };

    let json = serde_json::to_string(&response).expect("response should serialize");
    let decoded: ActionResponse = serde_json::from_str(&json).expect("response should decode");

    assert!(decoded.success);
    assert_eq!(decoded.request_id, request_id);
}
