use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpointProfile {
    pub id: String,
    pub name: String,
    pub network_name: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub notes: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalUserSummary {
    pub username: String,
    pub full_name: Option<String>,
    pub disabled: bool,
    pub password_required: bool,
    pub is_administrator: bool,
    pub active_session_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersResponse {
    pub users: Vec<LocalUserSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocalUserRequest {
    pub username: String,
    pub full_name: Option<String>,
    pub password: String,
    pub must_change_password: bool,
    pub enabled: bool,
    pub admin: bool,
    pub reason: Option<String>,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteLocalUserRequest {
    pub delete_profile: bool,
    pub logoff_sessions: bool,
    pub reason: Option<String>,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
    pub reason: Option<String>,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAccountEnabledRequest {
    pub enabled: bool,
    pub reason: Option<String>,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAdministratorRequest {
    pub enabled: bool,
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
pub struct ActionResponse {
    pub success: bool,
    pub message: String,
    pub request_id: Uuid,
}

pub type PasswordChangeResponse = ActionResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub request_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSessionSummary {
    pub session_id: u32,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub state: String,
    pub client_name: Option<String>,
    pub is_console: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<LocalSessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoffSessionRequest {
    pub reason: Option<String>,
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
