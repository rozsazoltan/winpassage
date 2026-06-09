use crate::audit::AuditWriter;
use crate::config::ServerConfig;
use anyhow::{Context, Result};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;
use winpassage_core::{validate_local_username, AuditEvent, AuditResult, PasswordPolicy};
use winpassage_protocol::{
    ActionResponse, ApiErrorResponse, ChangeOwnPasswordRequest, CreateLocalUserRequest,
    DeleteLocalUserRequest, HealthResponse, ListSessionsResponse, ListUsersResponse,
    LogoffSessionRequest, PasswordChangeResponse, ResetPasswordRequest, SetAccountEnabledRequest,
    SetAdministratorRequest,
};

#[derive(Clone)]
struct AppState {
    config: ServerConfig,
    audit: AuditWriter,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
    request_id: Option<Uuid>,
}

impl ApiError {
    fn bad_request(message: impl Into<String>, request_id: Option<Uuid>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            request_id,
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
            request_id: None,
        }
    }

    fn conflict(message: impl Into<String>, request_id: Option<Uuid>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
            request_id,
        }
    }

    fn internal(message: impl Into<String>, request_id: Option<Uuid>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
            request_id,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorResponse {
                error: self.message,
                request_id: self.request_id,
            }),
        )
            .into_response()
    }
}

pub async fn serve_until_shutdown(config: ServerConfig) -> Result<()> {
    validate_transport(&config)?;
    let audit = AuditWriter::spawn(config.audit_log.clone());
    let bind = config.bind;
    let app = router(AppState { config, audit });

    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("failed to bind WinPassage server on {bind}"))?;

    tracing::info!(%bind, "WinPassage server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

pub async fn serve_with_shutdown(
    config: ServerConfig,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    validate_transport(&config)?;
    let audit = AuditWriter::spawn(config.audit_log.clone());
    let bind = config.bind;
    let app = router(AppState { config, audit });

    let listener = TcpListener::bind(bind).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        let _ = shutdown.await;
    })
    .await?;

    Ok(())
}

fn validate_transport(config: &ServerConfig) -> Result<()> {
    if config.require_tls && !config.bind.ip().is_loopback() {
        anyhow::bail!(
            "WINPASSAGE_REQUIRE_TLS=true refuses a non-loopback plain HTTP listener; use 127.0.0.1, set up TLS/mTLS in front of the agent, or explicitly set WINPASSAGE_REQUIRE_TLS=false for an isolated private LAN"
        );
    }

    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/users", get(list_users).post(create_user))
        .route("/v1/users/{username}", delete(delete_user))
        .route(
            "/v1/users/{username}/password/reset",
            post(reset_password),
        )
        .route("/v1/users/{username}/enabled", post(set_user_enabled))
        .route("/v1/users/{username}/admin", post(set_user_administrator))
        .route("/v1/sessions", get(list_sessions))
        .route("/v1/sessions/{session_id}/logoff", post(logoff_session))
        .route("/v1/me/password/change", post(change_own_password))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "winpassage-server".to_string(),
    })
}

async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListUsersResponse>, ApiError> {
    require_admin(&state, &headers)?;

    let users = winpassage_windows::list_local_users()
        .map_err(|error| ApiError::internal(error.to_string(), None))?;

    Ok(Json(ListUsersResponse { users }))
}

async fn create_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<CreateLocalUserRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    require_admin(&state, &headers)?;
    validate_local_username(&request.username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;
    password_policy(&state)
        .validate(&request.password)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = (|| -> anyhow::Result<()> {
        winpassage_windows::create_local_user(
            &request.username,
            request.full_name.as_deref(),
            &request.password,
            request.must_change_password,
            request.enabled,
        )?;

        if request.admin {
            winpassage_windows::grant_local_administrator(&request.username)?;
        }

        Ok(())
    })();

    audit_admin_action(
        &state,
        addr,
        "admin_user_create",
        &request.username,
        result.as_ref().map(|_| ()).map_err(|error| error.to_string()),
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::internal(error.to_string(), Some(request_id)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: "local user created".to_string(),
        request_id,
    }))
}

async fn delete_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(request): Json<DeleteLocalUserRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    require_admin(&state, &headers)?;
    validate_local_username(&username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = (|| -> anyhow::Result<()> {
        if request.logoff_sessions {
            for session in winpassage_windows::list_sessions()? {
                if session
                    .username
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(&username))
                {
                    winpassage_windows::logoff_session(session.session_id)?;
                }
            }
        }

        if winpassage_windows::is_local_administrator(&username).unwrap_or(false)
            && winpassage_windows::local_administrator_count().unwrap_or(0) <= 1
        {
            anyhow::bail!("refusing to delete the last local administrator account");
        }

        winpassage_windows::delete_local_user(&username)?;

        if request.delete_profile {
            tracing::warn!(%username, "profile deletion was requested but is not implemented in this scaffold; delete the Windows profile with the supported profile API before enabling this option in production");
        }

        Ok(())
    })();

    let status = result.as_ref().map(|_| ()).map_err(|error| error.to_string());
    audit_admin_action(
        &state,
        addr,
        "admin_user_delete",
        &username,
        status,
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::conflict(error.to_string(), Some(request_id)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: "local user deleted".to_string(),
        request_id,
    }))
}

async fn reset_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<PasswordChangeResponse>, ApiError> {
    require_admin(&state, &headers)?;
    validate_local_username(&username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;
    password_policy(&state)
        .validate(&request.new_password)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = winpassage_windows::reset_local_user_password(&username, &request.new_password);

    audit_admin_action(
        &state,
        addr,
        "admin_password_reset",
        &username,
        result.as_ref().map(|_| ()).map_err(|error| error.to_string()),
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::internal(error.to_string(), Some(request_id)))?;

    Ok(Json(PasswordChangeResponse {
        success: true,
        message: "password reset completed".to_string(),
        request_id,
    }))
}

async fn set_user_enabled(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(request): Json<SetAccountEnabledRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    require_admin(&state, &headers)?;
    validate_local_username(&username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = winpassage_windows::set_local_user_enabled(&username, request.enabled);

    audit_admin_action(
        &state,
        addr,
        if request.enabled { "admin_user_enable" } else { "admin_user_disable" },
        &username,
        result.as_ref().map(|_| ()).map_err(|error| error.to_string()),
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::internal(error.to_string(), Some(request_id)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: if request.enabled {
            "local user enabled".to_string()
        } else {
            "local user disabled".to_string()
        },
        request_id,
    }))
}

async fn set_user_administrator(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(request): Json<SetAdministratorRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    require_admin(&state, &headers)?;
    validate_local_username(&username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = if request.enabled {
        winpassage_windows::grant_local_administrator(&username)
    } else {
        winpassage_windows::revoke_local_administrator(&username)
    };

    audit_admin_action(
        &state,
        addr,
        if request.enabled { "admin_grant" } else { "admin_revoke" },
        &username,
        result.as_ref().map(|_| ()).map_err(|error| error.to_string()),
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::conflict(error.to_string(), Some(request_id)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: if request.enabled {
            "administrator access granted".to_string()
        } else {
            "administrator access revoked".to_string()
        },
        request_id,
    }))
}

async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListSessionsResponse>, ApiError> {
    require_admin(&state, &headers)?;

    let sessions = winpassage_windows::list_sessions()
        .map_err(|error| ApiError::internal(error.to_string(), None))?;

    Ok(Json(ListSessionsResponse { sessions }))
}

async fn logoff_session(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(session_id): Path<u32>,
    Json(request): Json<LogoffSessionRequest>,
) -> Result<Json<ActionResponse>, ApiError> {
    require_admin(&state, &headers)?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = winpassage_windows::logoff_session(session_id);

    audit_admin_action(
        &state,
        addr,
        "admin_session_logoff",
        &format!("session:{session_id}"),
        result.as_ref().map(|_| ()).map_err(|error| error.to_string()),
        request.reason,
        request_id,
    );

    result.map_err(|error| ApiError::conflict(error.to_string(), Some(request_id)))?;

    Ok(Json(ActionResponse {
        success: true,
        message: "session logoff requested".to_string(),
        request_id,
    }))
}

async fn change_own_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<ChangeOwnPasswordRequest>,
) -> Result<Json<PasswordChangeResponse>, ApiError> {
    validate_local_username(&request.username)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;
    password_policy(&state)
        .validate(&request.new_password)
        .map_err(|error| ApiError::bad_request(error.to_string(), request.request_id))?;

    let request_id = request.request_id.unwrap_or_else(Uuid::new_v4);
    let result = winpassage_windows::change_own_password(
        &request.username,
        &request.current_password,
        &request.new_password,
    );

    let mut audit = AuditEvent::new(
        "self_password_change",
        request.username.clone(),
        request.username.clone(),
        if result.is_ok() {
            AuditResult::Success
        } else {
            AuditResult::Failure
        },
        request_id,
    );
    audit.source_ip = Some(addr.ip().to_string());
    audit.message = result.as_ref().err().map(|error| error.to_string());
    state.audit.write(audit);

    result.map_err(|error| ApiError::internal(error.to_string(), Some(request_id)))?;

    Ok(Json(PasswordChangeResponse {
        success: true,
        message: "password change completed".to_string(),
        request_id,
    }))
}

fn audit_admin_action(
    state: &AppState,
    addr: SocketAddr,
    event: &str,
    subject: &str,
    result: std::result::Result<(), String>,
    reason: Option<String>,
    request_id: Uuid,
) {
    let mut audit = AuditEvent::new(
        event,
        "admin",
        subject.to_string(),
        if result.is_ok() {
            AuditResult::Success
        } else {
            AuditResult::Failure
        },
        request_id,
    );
    audit.source_ip = Some(addr.ip().to_string());
    audit.reason = reason;
    audit.message = result.err();
    state.audit.write(audit);
}

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(expected) = state.config.admin_token.as_deref() else {
        return Err(ApiError::unauthorized("admin token is not configured"));
    };

    let Some(header) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(ApiError::unauthorized("missing Authorization header"));
    };

    let value = header
        .to_str()
        .map_err(|_| ApiError::unauthorized("invalid Authorization header"))?;

    let token = value.strip_prefix("Bearer ").unwrap_or(value);

    if token != expected {
        return Err(ApiError::unauthorized("invalid admin token"));
    }

    Ok(())
}

fn password_policy(state: &AppState) -> PasswordPolicy {
    PasswordPolicy {
        min_length: state.config.password_min_length,
        ..PasswordPolicy::default()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
