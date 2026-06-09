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
        Self::from_values(|key| env::var(key).ok())
    }

    pub fn from_env_with_overrides(
        bind: Option<SocketAddr>,
        admin_token: Option<String>,
        require_tls: Option<bool>,
    ) -> Result<Self> {
        let mut config = Self::from_env()?;

        if let Some(bind) = bind {
            config.bind = bind;
        }

        if let Some(admin_token) = admin_token {
            config.admin_token = Some(admin_token);
        }

        if let Some(require_tls) = require_tls {
            config.require_tls = require_tls;
        }

        Ok(config)
    }

    fn from_values(get: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let bind_value = get("WINPASSAGE_BIND").unwrap_or_else(|| "127.0.0.1:4487".to_string());
        let bind = bind_value
            .parse::<SocketAddr>()
            .with_context(|| format!("invalid WINPASSAGE_BIND value: {bind_value}"))?;

        let admin_token = get("WINPASSAGE_ADMIN_TOKEN").filter(|value| !value.trim().is_empty());

        let require_tls = get("WINPASSAGE_REQUIRE_TLS")
            .map(|value| !matches!(value.as_str(), "0" | "false" | "FALSE" | "False"))
            .unwrap_or(true);

        let audit_log = get("WINPASSAGE_AUDIT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(default_audit_log_path);

        let password_min_length = get("WINPASSAGE_PASSWORD_MIN_LENGTH")
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

#[cfg(test)]
mod tests {
    use super::ServerConfig;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn config_from(values: &[(&str, &str)]) -> ServerConfig {
        let values: HashMap<&str, &str> = values.iter().copied().collect();
        ServerConfig::from_values(|key| values.get(key).map(|value| (*value).to_string()))
            .expect("config should parse")
    }

    #[test]
    fn defaults_to_loopback_with_tls_required() {
        let config = config_from(&[]);

        assert_eq!(config.bind.to_string(), "127.0.0.1:4487");
        assert!(config.require_tls);
        assert_eq!(config.password_min_length, 12);
        assert!(config.admin_token.is_none());
    }

    #[test]
    fn parses_operator_overrides() {
        let config = config_from(&[
            ("WINPASSAGE_BIND", "0.0.0.0:4487"),
            ("WINPASSAGE_ADMIN_TOKEN", "dev-token"),
            ("WINPASSAGE_REQUIRE_TLS", "false"),
            (
                "WINPASSAGE_AUDIT_LOG",
                "C:/ProgramData/WinPassage/audit.jsonl",
            ),
            ("WINPASSAGE_PASSWORD_MIN_LENGTH", "16"),
        ]);

        assert_eq!(config.bind.to_string(), "0.0.0.0:4487");
        assert_eq!(config.admin_token.as_deref(), Some("dev-token"));
        assert!(!config.require_tls);
        assert_eq!(
            config.audit_log,
            PathBuf::from("C:/ProgramData/WinPassage/audit.jsonl")
        );
        assert_eq!(config.password_min_length, 16);
    }

    #[test]
    fn rejects_invalid_bind_address() {
        let values: HashMap<&str, &str> = [("WINPASSAGE_BIND", "not-a-socket")].into();
        let error =
            ServerConfig::from_values(|key| values.get(key).map(|value| (*value).to_string()))
                .expect_err("invalid bind address should fail");

        assert!(error.to_string().contains("invalid WINPASSAGE_BIND"));
    }

    #[test]
    fn ignores_blank_admin_token() {
        let config = config_from(&[("WINPASSAGE_ADMIN_TOKEN", "   ")]);
        assert!(config.admin_token.is_none());
    }
}
