//! Error types for rok-auth.

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AuthError {
    #[error("invalid or expired token")]
    InvalidToken,

    #[error("token has expired")]
    TokenExpired,

    #[error("insufficient permissions: required role `{0}`")]
    Forbidden(String),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("password hashing failed: {0}")]
    HashError(String),

    #[error("internal auth error: {0}")]
    Internal(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("account locked: {0}")]
    AccountLocked(String),

    #[error("invalid TOTP code")]
    InvalidTotp,

    #[error("user not found")]
    UserNotFound,

    #[error("email already exists")]
    EmailExists,

    #[error("invalid verification token")]
    InvalidVerificationToken,

    #[error("oauth error: {0}")]
    OAuthError(String),
}

impl AuthError {
    pub fn to_response(&self) -> AuthErrorResponse {
        use axum::http::StatusCode;

        let (status, code) = match self {
            AuthError::InvalidToken | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "INVALID_TOKEN")
            }
            AuthError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            AuthError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            AuthError::AccountLocked(_) => (StatusCode::FORBIDDEN, "ACCOUNT_LOCKED"),
            AuthError::InvalidTotp => (StatusCode::UNAUTHORIZED, "INVALID_TOTP"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
            AuthError::EmailExists => (StatusCode::CONFLICT, "EMAIL_EXISTS"),
            AuthError::InvalidVerificationToken => (StatusCode::BAD_REQUEST, "INVALID_TOKEN"),
            AuthError::OAuthError(_) => (StatusCode::BAD_REQUEST, "OAUTH_ERROR"),
            AuthError::HashError(_) | AuthError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };

        AuthErrorResponse {
            status_code: status.as_u16(),
            error_code: code.to_string(),
            message: self.to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct AuthErrorResponse {
    pub status_code: u16,
    pub error_code: String,
    pub message: String,
}

pub type AuthResult<T> = Result<T, AuthError>;

pub trait AuthResultExt<T> {
    fn map_auth_err(self) -> Self;
    fn inspect_auth_err(self, f: impl FnOnce(&AuthError)) -> Self;
}

impl<T> AuthResultExt<T> for Result<T, AuthError> {
    fn map_auth_err(self) -> Self {
        self.map_err(|e| e)
    }

    fn inspect_auth_err(self, f: impl FnOnce(&AuthError)) -> Self {
        self.inspect_err(f)
    }
}
