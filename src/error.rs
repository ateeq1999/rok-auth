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
}
