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
//! - **Rate Limiting** - Token bucket and sliding window rate limiting
//! - **Security Hardening** - Brute force detection, security headers
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
//! ## Builder Pattern
//!
//! ```rust,no_run
//! use rok_auth::{Auth, AuthConfigBuilder};
//!
//! let auth = Auth::new(
//!     AuthConfigBuilder::new()
//!         .secret("my-secret")
//!         .token_ttl_hours(2)
//!         .refresh_ttl_days(14)
//!         .issuer("my-app")
//!         .build()
//!         .unwrap()
//! );
//! ```
//!
//! ## Password Hashing
//!
//! ```rust,no_run
//! use rok_auth::password::{hash, verify};
//!
//! let hash = hash("password123").unwrap();
//! assert!(verify("password123", &hash).unwrap());
//! ```
//!
//! ## Rate Limiting
//!
//! ```rust,no_run
//! use rok_auth::security::{RateLimiter, RateLimitConfig};
//!
//! # async {
//! let config = RateLimitConfig {
//!     requests_per_window: 100,
//!     window_secs: 60,
//!     burst_size: 10,
//! };
//! let limiter = RateLimiter::new(config);
//!
//! let result = limiter.check_ip("192.168.1.1").await;
//! // Check if request is allowed
//! # };
//! ```
//!
//! ## Security Headers
//!
//! ```rust,no_run
//! use rok_auth::security::SecurityHeaders;
//!
//! let headers = SecurityHeaders::new()
//!     .strict();
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
//! | [`builders`] | Builder patterns for configuration |
//! | [`utils`] | Utility functions and extensions |
//! | [`security`] | Rate limiting and security hardening |
//! | [`authorization`] | RBAC and permissions |

/// Procedural macros re-exported for convenience.
///
/// Users only need to depend on `rok-auth` — there is no need to add
/// `rok-auth-macros` to `Cargo.toml` separately.
///
/// ```rust,ignore
/// use rok_auth::macros::{require_role, require_any_role, require_all_roles, require_fresh};
/// // or at the crate root:
/// use rok_auth::{require_role, require_any_role};
/// ```
pub use rok_auth_macros::{
    require_role, require_any_role, require_all_roles, require_fresh, UserProvider,
};

pub mod authorization;
pub mod builders;
pub mod claims;
pub mod config;
pub mod error;
pub mod jwt;
pub mod jwt_strict;
pub mod password;
pub mod providers;
pub mod security;
pub mod services;
pub mod session;
pub mod tokens;
pub mod utils;
pub mod web;

pub use authorization::{Permission, Policy, Role};
pub use builders::{AuthConfigBuilder, AuthConfigBuilderError};
pub use claims::{Claims, RefreshClaims};
pub use config::AuthConfig;
pub use error::{AuthError, AuthErrorResponse, AuthResult};
pub use jwt::Auth;
pub use jwt_strict::{StrictValidator, JwtAlgorithmType};
pub use password::{hash, verify};
pub use providers::UserProvider;
pub use security::{
    BruteForceDetector, CorsConfig, CsrfProtection, Device, DeviceManager, DeviceType,
    HealthCheck, HealthChecker, HealthState, HealthStatus, IpReputationChecker,
    MetricsCollector, MultiRateLimiter, RateLimitConfig, RateLimitResult, RateLimiter,
    SecurityAuditEvent, SecurityEventType, SecurityHeaders, SecurityWebhook, StepUpAuth,
    StuffingResult, StuffingReason, SuspiciousActivity, TokenBlacklist,
};
pub use session::SessionToken;
pub use services::{email, totp};
pub use tokens::{TokenAbility, TokenPair, TokenWithAbilities};
pub use utils::{
    auth_from_secret, auth_with_defaults, format_duration, parse_duration, random_secret, OptExt,
};
