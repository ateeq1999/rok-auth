//! Built-in authentication services.
//!
//! This module provides ready-to-use authentication services that can be
//! configured and integrated into your application.

mod session;
mod oauth;

pub use session::SessionService;
pub use oauth::{OAuthProvider, OAuthConfig};
