//! Builder patterns for configuration types.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AuthConfigBuilder {
    secret: Option<String>,
    token_ttl: Duration,
    refresh_ttl: Duration,
    issuer: Option<String>,
}

impl AuthConfigBuilder {
    pub fn new() -> Self {
        Self {
            secret: None,
            token_ttl: Duration::from_secs(3600),
            refresh_ttl: Duration::from_secs(60 * 60 * 24 * 7),
            issuer: None,
        }
    }

    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    pub fn token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = ttl;
        self
    }

    pub fn token_ttl_secs(mut self, secs: u64) -> Self {
        self.token_ttl = Duration::from_secs(secs);
        self
    }

    pub fn token_ttl_minutes(mut self, mins: u64) -> Self {
        self.token_ttl = Duration::from_secs(mins * 60);
        self
    }

    pub fn token_ttl_hours(mut self, hours: u64) -> Self {
        self.token_ttl = Duration::from_secs(hours * 3600);
        self
    }

    pub fn refresh_ttl(mut self, ttl: Duration) -> Self {
        self.refresh_ttl = ttl;
        self
    }

    pub fn refresh_ttl_days(mut self, days: u64) -> Self {
        self.refresh_ttl = Duration::from_secs(days * 24 * 3600);
        self
    }

    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    pub fn build(self) -> Result<crate::AuthConfig, AuthConfigBuilderError> {
        let secret = self.secret.ok_or(AuthConfigBuilderError::MissingSecret)?;
        if secret.is_empty() {
            return Err(AuthConfigBuilderError::EmptySecret);
        }
        Ok(crate::AuthConfig {
            secret,
            token_ttl: self.token_ttl,
            refresh_ttl: self.refresh_ttl,
            issuer: self.issuer,
        })
    }
}

impl Default for AuthConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthConfigBuilderError {
    #[error("secret is required")]
    MissingSecret,
    #[error("secret cannot be empty")]
    EmptySecret,
}

impl From<AuthConfigBuilderError> for crate::AuthError {
    fn from(e: AuthConfigBuilderError) -> Self {
        crate::AuthError::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_config_with_builder() {
        let config = AuthConfigBuilder::new()
            .secret("my-secret-key")
            .token_ttl_hours(2)
            .refresh_ttl_days(14)
            .issuer("my-app")
            .build()
            .unwrap();

        assert_eq!(config.secret, "my-secret-key");
        assert_eq!(config.token_ttl.as_secs(), 7200);
        assert_eq!(config.refresh_ttl.as_secs(), 14 * 24 * 3600);
        assert_eq!(config.issuer, Some("my-app".to_string()));
    }

    #[test]
    fn builder_requires_secret() {
        let result = AuthConfigBuilder::new().build();
        assert!(matches!(result, Err(AuthConfigBuilderError::MissingSecret)));
    }

    #[test]
    fn builder_rejects_empty_secret() {
        let result = AuthConfigBuilder::new().secret("").build();
        assert!(matches!(result, Err(AuthConfigBuilderError::EmptySecret)));
    }
}
