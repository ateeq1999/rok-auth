use std::time::Duration;

/// Configuration for [`crate::Auth`].
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// HMAC secret used to sign JWTs (HS256).
    pub secret: String,

    /// How long access tokens are valid.  Default: 1 hour.
    pub token_ttl: Duration,

    /// How long refresh tokens are valid.  Default: 7 days.
    pub refresh_ttl: Duration,

    /// Issuer claim (`iss`) embedded in every token.
    pub issuer: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            token_ttl: Duration::from_secs(3600),
            refresh_ttl: Duration::from_secs(60 * 60 * 24 * 7),
            issuer: None,
        }
    }
}
