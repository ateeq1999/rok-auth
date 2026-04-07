//! Axum integration for rok-auth.

mod error;
mod extractor;
mod guard;
mod layer;

pub use error::AuthErrorResponse;
pub use extractor::OptionalClaims;
pub use guard::{RequireRole, RoleMarker};
pub use layer::AuthLayer;

pub type Claims = crate::Claims;
