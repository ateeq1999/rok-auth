//! Axum extractors for JWT authentication.

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{Auth, AuthError, Claims};

pub struct OptionalClaims(pub Option<Claims>);

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth = parts
            .extensions
            .get::<Arc<Auth>>()
            .cloned()
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "configuration_error",
                        "message": "AuthLayer not installed"
                    })),
                )
                    .into_response()
            })?;

        let token = extract_bearer(parts).map_err(IntoResponse::into_response)?;
        auth.verify(&token).map_err(|e| e.into_response())
    }
}

impl<S> FromRequestParts<S> for OptionalClaims
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(auth) = parts.extensions.get::<Arc<Auth>>().cloned() else {
            return Ok(OptionalClaims(None));
        };

        let claims = extract_bearer(parts)
            .ok()
            .and_then(|token| auth.verify(&token).ok());

        Ok(OptionalClaims(claims))
    }
}

fn extract_bearer(parts: &Parts) -> Result<String, AuthError> {
    let header_val = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::InvalidToken)?;

    header_val
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or(AuthError::InvalidToken)
}
