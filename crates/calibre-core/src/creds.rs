//! Source selection, credential resolution, and timeout helpers.
//!
//! Rule: explicit CLI/MCP args win, environment is the fallback, and a
//! missing secret is a typed [`DeviceError::Auth`] -- never a panic, never a
//! log line containing the secret itself.
use crate::error::DeviceError;
use std::future::Future;
use std::time::Duration;

/// Default single-call I/O budget.
pub const DEVICE_TIMEOUT: Duration = Duration::from_secs(5);

/// Library source modes. `Demo` is the deterministic oracle for tests and probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    LocalSqlite,
    ContentServer,
    Demo,
}

impl SourceKind {
    /// Parse hostile input: case-insensitive, whitespace-tolerant, aliases accepted.
    pub fn parse(raw: &str) -> Result<Self, DeviceError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "sqlite" | "local" | "db" => Ok(Self::LocalSqlite),
            "server" | "opds" | "http" => Ok(Self::ContentServer),
            "demo" | "stub" | "mock" => Ok(Self::Demo),
            _ => Err(DeviceError::InvalidInput(format!(
                "unknown source '{raw}': expected local|server|demo"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalSqlite => "local",
            Self::ContentServer => "server",
            Self::Demo => "demo",
        }
    }
}

/// Optional account material for remote Calibre Content Server / OPDS endpoints.
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Credentials {
    /// Explicit args win; `CALIBRE_USERNAME` / `CALIBRE_PASSWORD` are fallbacks.
    pub fn resolve(
        kind: SourceKind,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self, DeviceError> {
        let clean = |v: Option<String>| v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let (env_user, env_pass) = match kind {
            SourceKind::ContentServer => (
                env_non_empty("CALIBRE_USERNAME"),
                env_non_empty("CALIBRE_PASSWORD"),
            ),
            SourceKind::LocalSqlite | SourceKind::Demo => (None, None),
        };
        Ok(Self {
            username: clean(username).or(env_user),
            password: clean(password).or(env_pass),
        })
    }
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Shared error classifier: timeout/auth/unsupported/protocol from message text.
pub fn classify_vendor_error(context: String) -> DeviceError {
    let lower = context.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") || lower.contains("deadline") {
        DeviceError::Timeout(context)
    } else if lower.contains("auth")
        || lower.contains("login")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("401")
        || lower.contains("403")
    {
        DeviceError::Auth(context)
    } else if lower.contains("unsupported") || lower.contains("not support") {
        DeviceError::Unsupported(context)
    } else {
        DeviceError::Protocol(context)
    }
}

/// Bound any single I/O op. Timeout carries the op name, never a secret.
pub async fn with_timeout<T>(op: &str, fut: impl Future<Output = T>) -> Result<T, DeviceError> {
    with_timeout_secs(op, DEVICE_TIMEOUT.as_secs(), fut).await
}

/// Explicit timeout budget in seconds.
pub async fn with_timeout_secs<T>(
    op: &str,
    secs: u64,
    fut: impl Future<Output = T>,
) -> Result<T, DeviceError> {
    match tokio::time::timeout(Duration::from_secs(secs), fut).await {
        Ok(value) => Ok(value),
        Err(_) => Err(DeviceError::Timeout(format!("{op} exceeded {secs}s"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_parse_accepts_aliases() {
        assert_eq!(SourceKind::parse("LOCAL"), Ok(SourceKind::LocalSqlite));
        assert_eq!(SourceKind::parse(" opds "), Ok(SourceKind::ContentServer));
        assert_eq!(SourceKind::parse("mock"), Ok(SourceKind::Demo));
        assert!(SourceKind::parse("unknown").is_err());
    }

    #[test]
    fn explicit_args_win_over_env() {
        let creds = Credentials::resolve(
            SourceKind::ContentServer,
            Some("reader".to_string()),
            Some("secret".to_string()),
        )
        .unwrap_or_default();
        assert_eq!(creds.username.as_deref(), Some("reader"));
        assert_eq!(creds.password.as_deref(), Some("secret"));
    }

    #[test]
    fn demo_needs_no_account() {
        let creds = Credentials::resolve(SourceKind::Demo, None, None).unwrap_or_default();
        assert!(creds.username.is_none());
    }
}
