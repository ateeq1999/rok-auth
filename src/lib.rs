//! # rok-auth
//!
//! JWT authentication and RBAC for the rok ecosystem.
//!
//! Inspired by [better-auth](https://better-auth.com/), rok-auth provides
//! built-in authentication services that integrate seamlessly with your
//! application once configured.
//!
//! ## Features
//!
//! - **JWT Authentication** - Secure token-based auth with access/refresh tokens
//! - **Password Hashing** - Argon2id password hashing
//! - **Session Management** - Cryptographically secure session tokens
//! - **RBAC** - Role-based access control with flexible role checking
//! - **OAuth Integration** - Built-in OAuth provider support
//! - **Two-Factor Authentication** - TOTP support
//! - **Email Verification** - Account verification flows
//! - **Web Framework Integration** - First-class Axum support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rok_auth::{Auth, AuthConfig, Claims};
//!
//! let auth = Auth::new(AuthConfig {
//!     secret: "super-secret-key".to_string(),
//!     ..Default::default()
//! });
//!
//! // Sign a token
//! let token = auth.sign(&Claims::new("user-123", vec!["admin"])).unwrap();
//!
//! // Verify and decode
//! let claims = auth.verify(&token).unwrap();
//! assert_eq!(claims.sub, "user-123");
//! assert!(claims.has_role("admin"));
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`claims`] | JWT claims structures |
//! | [`config`] | Authentication configuration |
//! | [`error`] | Error types |
//! | [`jwt`] | JWT signing and verification |
//! | [`password`] | Password hashing with Argon2id |
//! | [`providers`] | User provider abstractions |
//! | [`session`] | Session token management |
//! | [`tokens`] | Token pair management |
//! | [`web`] | Web framework integrations |
//! | [`services`] | Built-in authentication services |

pub mod authorization;
pub mod claims;
pub mod config;
pub mod error;
pub mod jwt;
pub mod password;
pub mod providers;
pub mod session;
pub mod tokens;
pub mod web;
pub mod services;

pub use claims::{Claims, RefreshClaims};
pub use config::AuthConfig;
pub use error::AuthError;
pub use jwt::Auth;
pub use providers::UserProvider;
pub use session::SessionToken;
pub use tokens::TokenPair;
