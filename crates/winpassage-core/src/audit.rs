use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
    pub actor: String,
    pub subject: String,
    pub result: AuditResult,
    pub request_id: Uuid,
    pub source_ip: Option<String>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

impl AuditEvent {
    pub fn new(
        event: impl Into<String>,
        actor: impl Into<String>,
        subject: impl Into<String>,
        result: AuditResult,
        request_id: Uuid,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event: event.into(),
            actor: actor.into(),
            subject: subject.into(),
            result,
            request_id,
            source_ip: None,
            reason: None,
            message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditEvent, AuditResult};
    use uuid::Uuid;

    #[test]
    fn audit_event_serializes_without_password_fields() {
        let event = AuditEvent::new(
            "admin_password_reset",
            "admin",
            "julia",
            AuditResult::Success,
            Uuid::nil(),
        );

        let json = serde_json::to_string(&event).expect("audit event should serialize");
        assert!(json.contains("admin_password_reset"));
        assert!(!json.to_ascii_lowercase().contains("password\":"));
        assert!(!json.contains("secret"));
    }
}
