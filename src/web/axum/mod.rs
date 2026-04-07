//! Axum integration for rok-auth.

mod layer;
mod extractor;
mod guard;
mod error;

pub use layer::AuthLayer;
pub use extractor::OptionalClaims;
pub use guard::{RequireRole, RoleMarker};
pub use error::AuthErrorResponse;

pub type Claims = crate::Claims;
