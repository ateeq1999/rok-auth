//! Axum response conversion for AuthError.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::AuthError;

pub struct AuthErrorResponse {
    pub code: &'static str,
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired"),
            AuthError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::HashError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "hash_error"),
            AuthError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            AuthError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            AuthError::AccountLocked(_) => (StatusCode::FORBIDDEN, "account_locked"),
            AuthError::InvalidTotp => (StatusCode::UNAUTHORIZED, "invalid_totp"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::EmailExists => (StatusCode::CONFLICT, "email_exists"),
            AuthError::InvalidVerificationToken => {
                (StatusCode::BAD_REQUEST, "invalid_verification_token")
            }
            AuthError::OAuthError(_) => (StatusCode::BAD_REQUEST, "oauth_error"),
        };

        (
            status,
            Json(json!({ "error": code, "message": self.to_string() })),
        )
            .into_response()
    }
}
