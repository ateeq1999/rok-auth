# rok-auth Developer Guide

rok-auth is a production-ready JWT authentication and RBAC library for Rust, inspired by [better-auth](https://better-auth.com/). It provides everything needed for authentication: JWT signing/verification, password hashing, session tokens, TOTP, OAuth, email verification, RBAC, rate limiting, and Axum integration — all in one crate.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Module Reference](#module-reference)
- [Setup & Build](#setup--build)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)

---

## Quick Start

```rust
use rok_auth::{Auth, AuthConfig, Claims};

// Create auth handle
let auth = Auth::new(AuthConfig {
    secret: "your-secret-key".to_string(),
    ..Default::default()
});

// Sign an access token
let token = auth.sign(&Claims::new("user-123", vec!["admin"])).unwrap();

// Verify and decode
let claims = auth.verify(&token).unwrap();
assert_eq!(claims.sub, "user-123");
assert!(claims.has_role("admin"));

// Issue and exchange refresh tokens
let refresh = auth.sign_refresh("user-123").unwrap();
let (new_access, new_refresh) = auth.exchange(&refresh).unwrap();
```

### Builder Pattern

```rust
use rok_auth::{Auth, AuthConfigBuilder};

let auth = Auth::new(
    AuthConfigBuilder::new()
        .secret("my-secret")
        .token_ttl_hours(2)
        .refresh_ttl_days(14)
        .issuer("my-app")
        .build()
        .unwrap(),
);
```

### One-liner helpers

```rust
use rok_auth::utils::{auth_from_secret, auth_with_defaults, random_secret};

let auth = auth_from_secret("my-secret").unwrap();   // from a known secret
let auth = auth_with_defaults().unwrap();             // random 256-bit secret
let secret = random_secret();                         // 64-char hex string
```

---

## Project Structure

```
rok-auth/
├── src/
│   ├── lib.rs               # Public API re-exports
│   ├── main.rs              # CLI binary entry point
│   ├── claims.rs            # Claims, RefreshClaims
│   ├── config.rs            # AuthConfig
│   ├── error.rs             # AuthError, AuthResult, AuthErrorResponse
│   ├── jwt.rs               # Auth — sign / verify / exchange
│   ├── jwt_strict.rs        # StrictValidator — algorithm enforcement
│   ├── builders.rs          # AuthConfigBuilder
│   ├── utils.rs             # auth_from_secret, random_secret, parse_duration, OptExt
│   ├── blacklist.rs         # TokenBlacklist — in-memory token revocation
│   ├── password/
│   │   └── hash.rs          # hash(), verify(), hash_async(), verify_async()
│   ├── providers/
│   │   └── trait_.rs        # UserProvider trait
│   ├── session/
│   │   └── token.rs         # SessionToken — 256-bit random hex
│   ├── tokens/
│   │   ├── pair.rs          # TokenPair — (access, refresh) bundle
│   │   ├── refresh.rs       # RefreshClaims re-export
│   │   └── abilities.rs     # TokenAbility, TokenWithAbilities
│   ├── services/
│   │   ├── totp.rs          # TotpService, BackupCodes
│   │   ├── oauth.rs         # OAuthService, OAuthProvider
│   │   └── email/           # EmailVerificationService, PasswordResetService
│   ├── authorization/
│   │   ├── roles.rs         # Role, RoleHierarchy, RoleManager
│   │   ├── permissions.rs   # Permission, PermissionScope
│   │   ├── policies.rs      # Policy, PolicyEvaluator
│   │   └── audit.rs         # AuditEvent, AuditLogger trait
│   ├── security/
│   │   ├── rate_limiter.rs  # RateLimiter, MultiRateLimiter
│   │   ├── headers.rs       # SecurityHeaders
│   │   ├── cors.rs          # CorsConfig
│   │   ├── csrf.rs          # CsrfProtection
│   │   ├── detection.rs     # BruteForceDetector, IpReputationChecker, CredentialStuffingDetector
│   │   ├── device.rs        # DeviceManager, Device, DeviceType
│   │   ├── health.rs        # HealthChecker, MetricsCollector
│   │   ├── step_up.rs       # StepUpAuth
│   │   └── webhook.rs       # SecurityWebhook, SecurityAuditEvent
│   └── web/axum/
│       ├── layer.rs         # AuthLayer — Axum middleware
│       ├── extractor.rs     # Claims, OptionalClaims extractors
│       ├── guard.rs         # Role guard
│       └── error.rs         # IntoResponse for AuthError
└── rok-auth-macros/
    └── src/lib.rs           # #[require_role], #[require_any_role] proc macros
```

---

## Module Reference

| Module | Key Types | Purpose |
|--------|-----------|---------|
| `jwt` | `Auth` | Sign/verify JWTs, refresh tokens, token exchange |
| `claims` | `Claims`, `RefreshClaims` | JWT payload structures |
| `config` | `AuthConfig` | Secret, TTLs, issuer |
| `error` | `AuthError`, `AuthResult` | Typed errors with HTTP status mapping |
| `builders` | `AuthConfigBuilder` | Fluent config construction |
| `utils` | `auth_from_secret`, `random_secret`, `parse_duration`, `OptExt` | Helpers |
| `password` | `hash`, `verify`, `hash_async`, `verify_async` | Argon2id hashing |
| `session` | `SessionToken` | 256-bit random session tokens |
| `tokens` | `TokenPair`, `TokenAbility`, `TokenWithAbilities` | Token bundles and scoped abilities |
| `authorization` | `Role`, `RoleManager`, `Permission`, `Policy` | RBAC, permissions, policies |
| `security` | `RateLimiter`, `BruteForceDetector`, `SecurityHeaders`, etc. | Rate limiting, hardening |
| `services::totp` | `TotpService`, `BackupCodes` | TOTP 2FA |
| `services::oauth` | `OAuthService` | OAuth provider integration |
| `services::email` | `EmailVerificationService`, `PasswordResetService` | Email flows |
| `web::axum` | `AuthLayer`, `Claims` extractor, `OptionalClaims` | Axum middleware + extractors |

---

## Setup & Build

**Prerequisites:** Rust 1.75+

```bash
cargo build                      # compile
cargo test                       # run all tests
cargo check                      # type-check without linking
cargo fmt                        # format
cargo clippy -- -D warnings      # lint
cargo run --bin rok-auth         # run the CLI binary
```

---

## Development Workflow

1. Read the relevant phase doc in `docs/`
2. Implement the feature; keep every file under **400 lines**
3. Write inline tests in `#[cfg(test)]` blocks
4. Run `cargo test && cargo clippy -- -D warnings`
5. Commit with a clear message:

```
feat: add TOTP backup code verification
fix: handle expired refresh tokens correctly
docs: rewrite phase-03 with complete API reference
```

---

## Code Style

| Rule | Detail |
|------|--------|
| File size | ≤ 400 lines per file |
| Errors | `thiserror` everywhere; no `.unwrap()` in library code |
| Async | CPU-heavy work (Argon2) → `spawn_blocking` |
| Tests | Every public function needs at least one test |
| Docs | All `pub` items need doc comments |
| Security | Never log secrets, tokens, or password hashes |

### Module template

```rust
//! Brief description.
//!
//! # Example
//!
//! ```rust,no_run
//! use rok_auth::MyType;
//! // ...
//! ```

// implementation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_case() { /* ... */ }
}
```
