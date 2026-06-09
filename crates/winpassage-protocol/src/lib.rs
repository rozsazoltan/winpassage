use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUserSummary {
    pub username: String,
    pub full_name: Option<String>,
    pub disabled: bool,
    pub password_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersResponse {
    pub users: Vec<LocalUserSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
    pub reason: Option<String>,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeOwnPasswordRequest {
    pub username: String,
    pub current_password: String,
    pub new_password: String,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChangeResponse {
    pub success: bool,
    pub message: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveMapping {
    pub letter: String,
    pub remote_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectMappedDrivesRequest {
    pub username: String,
    pub password: String,
    pub drives: Vec<DriveMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveReconnectResult {
    pub letter: String,
    pub remote_path: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectMappedDrivesResponse {
    pub results: Vec<DriveReconnectResult>,
}
