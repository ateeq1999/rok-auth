//! Refresh token claims.

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub typ: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
}

impl RefreshClaims {
    pub fn new(sub: impl Into<String>, exp: i64, iss: Option<String>) -> Self {
        Self {
            sub: sub.into(),
            typ: "refresh".to_string(),
            exp,
            iat: Utc::now().timestamp(),
            iss,
        }
    }
}
