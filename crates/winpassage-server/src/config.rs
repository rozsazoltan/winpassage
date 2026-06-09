use anyhow::{Context, Result};
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    pub admin_token: Option<String>,
    pub require_tls: bool,
    pub audit_log: PathBuf,
    pub password_min_length: usize,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        let bind = env::var("WINPASSAGE_BIND").unwrap_or_else(|_| "127.0.0.1:4487".to_string());
        let bind = bind
            .parse::<SocketAddr>()
            .with_context(|| format!("invalid WINPASSAGE_BIND value: {bind}"))?;

        let admin_token = env::var("WINPASSAGE_ADMIN_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty());

        let require_tls = env::var("WINPASSAGE_REQUIRE_TLS")
            .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "False"))
            .unwrap_or(true);

        let audit_log = env::var("WINPASSAGE_AUDIT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_audit_log_path());

        let password_min_length = env::var("WINPASSAGE_PASSWORD_MIN_LENGTH")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(12);

        Ok(Self {
            bind,
            admin_token,
            require_tls,
            audit_log,
            password_min_length,
        })
    }
}

fn default_audit_log_path() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(r"C:\ProgramData\WinPassage\audit.jsonl")
    }

    #[cfg(not(windows))]
    {
        PathBuf::from("./winpassage-audit.jsonl")
    }
}
