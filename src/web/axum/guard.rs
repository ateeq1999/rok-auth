//! Role-based access guard extractors.

use std::marker::PhantomData;

use axum::{extract::FromRequestParts, http::request::Parts, response::Response};

use super::extractor::OptionalClaims;
use crate::AuthError;

pub trait RoleMarker: Send + Sync + 'static {
    const ROLE: &'static str;
}

pub struct RequireRole<R: RoleMarker>(PhantomData<R>);

impl<S, R: RoleMarker> FromRequestParts<S> for RequireRole<R>
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::response::IntoResponse;

        let OptionalClaims(maybe) = OptionalClaims::from_request_parts(parts, state)
            .await
            .unwrap();

        let claims = maybe.ok_or_else(|| AuthError::InvalidToken.into_response())?;

        if !claims.has_role(R::ROLE) {
            return Err(AuthError::Forbidden(R::ROLE.to_string()).into_response());
        }

        Ok(RequireRole(PhantomData))
    }
}
