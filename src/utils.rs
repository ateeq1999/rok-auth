//! Utility functions for rok-auth.

use crate::{Auth, AuthError};
use std::time::Duration;

pub use crate::builders::AuthConfigBuilder;

pub fn auth_from_secret(secret: &str) -> Result<Auth, AuthError> {
    let config = AuthConfigBuilder::new()
        .secret(secret)
        .build()
        .map_err(|e| AuthError::Internal(e.to_string()))?;
    Ok(Auth::new(config))
}

pub fn auth_with_defaults() -> Result<Auth, AuthError> {
    auth_from_secret(&random_secret())
}

pub fn random_secret() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn parse_duration(s: &str) -> Result<Duration, ParseDurationError> {
    let s = s.trim();

    if let Some(n) = s.strip_suffix("s") {
        let n: u64 = n.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
        return Ok(Duration::from_secs(n));
    }

    if let Some(n) = s.strip_suffix("m") {
        let n: u64 = n.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
        return Ok(Duration::from_secs(n * 60));
    }

    if let Some(n) = s.strip_suffix("h") {
        let n: u64 = n.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
        return Ok(Duration::from_secs(n * 3600));
    }

    if let Some(n) = s.strip_suffix("d") {
        let n: u64 = n.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
        return Ok(Duration::from_secs(n * 24 * 3600));
    }

    if let Some(n) = s.strip_suffix("w") {
        let n: u64 = n.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
        return Ok(Duration::from_secs(n * 7 * 24 * 3600));
    }

    let secs: u64 = s.parse().map_err(|_| ParseDurationError::InvalidNumber)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Debug, thiserror::Error)]
pub enum ParseDurationError {
    #[error("invalid number format")]
    InvalidNumber,
}

pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs >= 7 * 24 * 3600 {
        let weeks = total_secs / (7 * 24 * 3600);
        format!("{}w", weeks)
    } else if total_secs >= 24 * 3600 {
        let days = total_secs / (24 * 3600);
        format!("{}d", days)
    } else if total_secs >= 3600 {
        let hours = total_secs / 3600;
        format!("{}h", hours)
    } else if total_secs >= 60 {
        let mins = total_secs / 60;
        format!("{}m", mins)
    } else {
        format!("{}s", total_secs)
    }
}

#[derive(Debug)]
pub struct NoneError;

impl std::fmt::Display for NoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "found None")
    }
}

impl std::error::Error for NoneError {}

pub trait OptExt<T> {
    fn ok_or_auth_error(self) -> Result<T, AuthError>;
}

impl<T> OptExt<T> for Option<T> {
    fn ok_or_auth_error(self) -> Result<T, AuthError> {
        self.ok_or_else(|| AuthError::Internal("unexpected None".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_formats() {
        assert_eq!(parse_duration("60s").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("1w").unwrap(), Duration::from_secs(604800));
        assert_eq!(parse_duration("3600").unwrap(), Duration::from_secs(3600));
    }

    #[test]
    fn format_duration_formats() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(86400)), "1d");
        assert_eq!(format_duration(Duration::from_secs(604800)), "1w");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m");
    }

    #[test]
    fn test_auth_from_secret() {
        let auth = auth_from_secret("test-secret").unwrap();
        let token = auth
            .sign(&crate::Claims::new("user", vec!["admin"]))
            .unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn random_secret_length() {
        let secret = random_secret();
        assert_eq!(secret.len(), 64);
    }
}
