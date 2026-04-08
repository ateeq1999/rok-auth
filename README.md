# rok-auth

A production-ready authentication library for Rust. JWT, RBAC, OAuth, 2FA, email verification, rate limiting, and Axum integration — all in one cohesive library.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## Features

| Feature | Description |
|---------|-------------|
| **JWT Authentication** | HS256 access/refresh tokens with configurable TTLs and issuer |
| **Password Hashing** | Argon2id with random salts, sync and async APIs |
| **Session Tokens** | 256-bit cryptographically random session tokens |
| **TOTP 2FA** | RFC 6238 time-based OTP with QR code URI and backup codes |
| **OAuth** | Social login with Google, GitHub, Discord, or custom providers |
| **Email Flows** | Account verification and password reset with pluggable senders |
| **RBAC** | Role hierarchy with inheritance, permissions, and policy evaluation |
| **Rate Limiting** | Token bucket + sliding window, per-IP/user/endpoint |
| **Security Hardening** | Brute-force detection, IP reputation, credential stuffing detection |
| **Security Headers** | HSTS, CSP, X-Frame-Options, and more — configurable presets |
| **CSRF Protection** | Token-based CSRF for SPA / cookie auth flows |
| **Axum Integration** | Middleware layer, `Claims` extractor, `OptionalClaims`, role guards |
| **Proc Macros** | `#[require_role]` and `#[require_any_role]` for handler guards |
| **Audit Logging** | Structured audit events with a pluggable logger trait |

---

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
rok-auth = { path = "." }
```

### Sign and verify a JWT

```rust
use rok_auth::{Auth, AuthConfig, Claims};

let auth = Auth::new(AuthConfig {
    secret: "your-secret-key-at-least-32-chars".to_string(),
    ..Default::default()
});

// Sign an access token
let token = auth.sign(&Claims::new("user-123", vec!["admin", "user"]))?;

// Verify and decode
let claims = auth.verify(&token)?;
assert_eq!(claims.sub, "user-123");
assert!(claims.has_role("admin"));
```

### Builder pattern

```rust
use rok_auth::{Auth, AuthConfigBuilder};

let auth = Auth::new(
    AuthConfigBuilder::new()
        .secret("my-secret")
        .token_ttl_hours(2)
        .refresh_ttl_days(14)
        .issuer("my-app")
        .build()?,
);
```

### One-liners

```rust
use rok_auth::utils::{auth_from_secret, random_secret};

let auth = auth_from_secret("my-secret")?;
let secret = random_secret(); // cryptographically random 64-char hex string
```

---

## Password Hashing

```rust
use rok_auth::password::{hash, verify, hash_async, verify_async};

// Sync
let h = hash("hunter2")?;
assert!(verify("hunter2", &h)?);

// Async (non-blocking, use in async handlers)
let h = hash_async("hunter2".to_string()).await?;
let ok = verify_async("hunter2".to_string(), h).await?;
```

---

## Refresh Token Flow

```rust
// Issue a refresh token
let refresh = auth.sign_refresh("user-123")?;

// Exchange for a new (access, refresh) pair
let (new_access, new_refresh) = auth.exchange(&refresh)?;
```

---

## Two-Factor Authentication (TOTP)

```rust
use rok_auth::services::totp::{TotpService, TotpConfig};

let service = TotpService::new(TotpConfig::default());

// 1. Enrollment: generate a secret and show a QR code
let secret = service.generate_secret();
let uri = service.provisioning_uri(&secret, "alice@example.com", "MyApp");
// Render uri as a QR code

// 2. Verification: check the code the user submitted
let valid = service.verify_code(&secret, "482913", 1)?; // tolerance = ±1 step

// Backup codes
use rok_auth::services::totp::BackupCodes;
let mut codes = BackupCodes::generate(10);
let used = codes.verify("a1b2c3d4e5f6g7h8"); // true on first use
```

---

## RBAC

```rust
use rok_auth::authorization::roles::RoleManager;

// Built-in hierarchy: superadmin ← admin ← moderator ← user, guest
let manager = RoleManager::new().with_default_roles();

let user_roles = vec!["admin".to_string()];
assert!(manager.check_role(&user_roles, "superadmin")); // inherited
assert!(!manager.check_role(&user_roles, "invalid"));
```

### Role guards via macros

```rust
use rok_auth_macros::require_role;
use rok_auth::Claims;

#[require_role("admin")]
async fn admin_only(claims: Claims) -> impl IntoResponse {
    format!("Hello, {}", claims.sub)
}
```

---

## Rate Limiting

```rust
use rok_auth::{RateLimiter, RateLimitConfig, RateLimitResult};

let limiter = RateLimiter::new(RateLimitConfig {
    requests_per_window: 60,
    window_secs: 60,
    burst_size: 5,
});

match limiter.check_ip("1.2.3.4").await {
    RateLimitResult::Allowed => { /* proceed */ }
    RateLimitResult::RateLimited { retry_after_secs } => {
        // return 429 with Retry-After header
    }
}
```

---

## Brute Force Protection

```rust
use rok_auth::BruteForceDetector;

let detector = BruteForceDetector::default(); // 5 attempts, 15-min lockout

match detector.record_failed_attempt("alice@example.com").await {
    AttemptResult::Allowed { attempts_remaining } => { /* warn user */ }
    AttemptResult::Locked { remaining_secs } => {
        return Err(AuthError::AccountLocked(format!("{}s remaining", remaining_secs)));
    }
}

// Clear on successful login
detector.record_successful_attempt("alice@example.com").await;
```

---

## Security Headers

```rust
use rok_auth::SecurityHeaders;

let headers = SecurityHeaders::new();         // sensible defaults
let headers = SecurityHeaders::new().strict(); // maximum security preset
let headers = SecurityHeaders::new()
    .with_hsts(63_072_000, true)
    .with_csp("default-src 'self'");
```

---

## Axum Integration

```rust
use axum::{Router, routing::get};
use rok_auth::{Auth, AuthConfig, Claims};
use rok_auth::web::axum::AuthLayer;
use std::sync::Arc;

let auth = Arc::new(Auth::new(AuthConfig {
    secret: "my-secret".to_string(),
    ..Default::default()
}));

let app = Router::new()
    .route("/protected", get(protected))
    .layer(AuthLayer::new(auth));

// Claims is extracted automatically from the Authorization: Bearer <token> header
async fn protected(claims: Claims) -> String {
    format!("Hello, {}", claims.sub)
}
```

### Optional authentication

```rust
use rok_auth::web::axum::OptionalClaims;

async fn public_or_private(OptionalClaims(claims): OptionalClaims) -> String {
    match claims {
        Some(c) => format!("logged in as {}", c.sub),
        None    => "anonymous".to_string(),
    }
}
```

---

## Error Handling

All functions return `Result<T, AuthError>`. Every variant maps to an HTTP status:

| Error | Status |
|-------|--------|
| `InvalidToken` / `TokenExpired` | 401 |
| `Forbidden` | 403 |
| `InvalidCredentials` | 401 |
| `RateLimited` | 429 |
| `AccountLocked` | 403 |
| `UserNotFound` | 404 |
| `EmailExists` | 409 |
| `InvalidTotp` | 401 |
| `Internal` / `HashError` | 500 |

```rust
// Convert to an HTTP-ready response
let response = err.to_response();
// AuthErrorResponse { status_code: 401, error_code: "INVALID_TOKEN", message: "..." }
```

`AuthError` implements `IntoResponse` for Axum — return it directly from handlers.

---

## Documentation Site

rok-auth ships a built-in documentation server:

```bash
cargo run --bin rok-auth
# → http://localhost:4000
```

The site renders all Markdown files in `docs/` and supports light/dark theme switching.

---

## Project Structure

```
src/
├── lib.rs               # Public API re-exports
├── main.rs              # Documentation site binary
├── claims.rs            # Claims, RefreshClaims
├── config.rs            # AuthConfig
├── error.rs             # AuthError, AuthResult
├── jwt.rs               # Auth — sign / verify / exchange
├── jwt_strict.rs        # StrictValidator — algorithm enforcement
├── builders.rs          # AuthConfigBuilder
├── utils.rs             # auth_from_secret, random_secret, parse_duration
├── blacklist.rs         # TokenBlacklist — in-memory revocation
├── password/            # Argon2id hashing
├── session/             # SessionToken
├── tokens/              # TokenPair, TokenAbility
├── services/            # TOTP, OAuth, Email
├── authorization/       # Roles, Permissions, Policies, Audit
├── security/            # Rate limiting, Headers, Detection, CSRF, Webhooks
└── web/axum/            # AuthLayer, extractors, guards

rok-auth-macros/         # #[require_role], #[require_any_role]
docs/                    # Phase-by-phase documentation
```

---

## Building & Testing

```bash
cargo build                      # compile
cargo test                       # 128 tests
cargo check                      # type-check
cargo clippy -- -D warnings      # lint
cargo fmt                        # format
cargo run --bin rok-auth         # start docs server on :4000
```

---

## Roadmap

- [x] JWT authentication (access + refresh tokens)
- [x] Argon2id password hashing
- [x] Session token management
- [x] TOTP two-factor authentication
- [x] OAuth integration (Google, GitHub, Discord)
- [x] Email verification & password reset
- [x] Role-based access control with hierarchy
- [x] Rate limiting (token bucket + sliding window)
- [x] Security hardening (brute force, IP reputation, headers)
- [x] Axum middleware and extractors
- [x] Procedural macros for role guards
- [ ] `rok-cli` standalone CLI tool
- [ ] Database-backed persistence layer
- [ ] PostgreSQL adapter via SQLx

---

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

at your option.
