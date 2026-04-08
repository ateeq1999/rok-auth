# Phase 1: Core Authentication Foundation

JWT signing/verification, configuration, claims, and error handling — the foundation everything else builds on.

---

## AuthConfig

Defined in [src/config.rs](../src/config.rs).

```rust
pub struct AuthConfig {
    pub secret: String,         // HMAC secret for HS256 signing — must not be empty
    pub token_ttl: Duration,    // Access token lifetime (default: 1 hour)
    pub refresh_ttl: Duration,  // Refresh token lifetime (default: 7 days)
    pub issuer: Option<String>, // `iss` claim embedded in every token
}
```

`AuthConfig` implements `Default` with a blank secret (which will panic at `Auth::new`). Always set a secret.

---

## Auth

Defined in [src/jwt.rs](../src/jwt.rs). The main handle — wraps encoding/decoding keys derived from `AuthConfig`.

### Construction

```rust
use rok_auth::{Auth, AuthConfig};

let auth = Auth::new(AuthConfig {
    secret: "at-least-32-chars-recommended".to_string(),
    ..Default::default()
});
```

Panics if `secret` is empty.

### Methods

#### `sign(&self, claims: &Claims) -> Result<String, AuthError>`

Signs an access token. Overwrites `iat`/`exp` from `token_ttl` and injects `iss` from config if not set on the claims.

```rust
let token = auth.sign(&Claims::new("user-123", vec!["admin", "user"]))?;
```

#### `verify(&self, token: &str) -> Result<Claims, AuthError>`

Validates signature, expiry, and issuer (if configured). Returns `AuthError::TokenExpired` for an expired token, `AuthError::InvalidToken` for everything else.

```rust
let claims = auth.verify(&token)?;
println!("subject: {}", claims.sub);
```

#### `sign_refresh(&self, subject: &str) -> Result<String, AuthError>`

Issues a refresh token for `subject` with a `typ: "refresh"` discriminator, valid for `refresh_ttl`.

#### `verify_refresh(&self, token: &str) -> Result<RefreshClaims, AuthError>`

Validates a refresh token. Returns `AuthError::InvalidToken` if the `typ` field is not `"refresh"` — so access tokens cannot be used as refresh tokens and vice-versa.

#### `exchange(&self, refresh_token: &str) -> Result<(String, String), AuthError>`

Verifies the refresh token and issues a fresh `(access_token, refresh_token)` pair. The new access token carries empty roles; callers should look up the user and call `sign` directly if roles are needed.

```rust
let (access, refresh) = auth.exchange(&old_refresh)?;
```

#### `config(&self) -> &AuthConfig`

Returns a reference to the underlying config.

---

## Claims

Defined in [src/claims.rs](../src/claims.rs).

```rust
pub struct Claims {
    pub sub: String,            // Subject — typically a user ID
    pub roles: Vec<String>,     // Roles assigned to this subject
    pub exp: i64,               // Expiry (Unix timestamp)
    pub iat: i64,               // Issued-at (Unix timestamp)
    pub iss: Option<String>,    // Issuer (optional)
}
```

### Construction

```rust
let claims = Claims::new("user-123", vec!["admin", "user"]);
```

Sets `iat` to now, `exp` to now + 1 hour. `Auth::sign` overrides `iat`/`exp` using `config.token_ttl`.

### Role checks

```rust
claims.has_role("admin")                    // true if "admin" is in roles
claims.has_any_role(&["admin", "editor"])   // true if any match
claims.has_all_roles(&["editor", "viewer"]) // true if all match
claims.is_valid()                           // true if not expired
```

---

## RefreshClaims

```rust
pub struct RefreshClaims {
    pub sub: String,
    pub typ: String,            // Always "refresh"
    pub exp: i64,
    pub iat: i64,
    pub iss: Option<String>,
}
```

---

## AuthError

Defined in [src/error.rs](../src/error.rs).

| Variant | HTTP Status | When |
|---------|-------------|------|
| `InvalidToken` | 401 | Malformed, wrong secret, or wrong type |
| `TokenExpired` | 401 | `exp` claim is in the past |
| `Forbidden(role)` | 403 | Missing required role |
| `InvalidCredentials` | 401 | Bad username/password |
| `HashError(msg)` | 500 | Argon2 failure |
| `Internal(msg)` | 500 | Unexpected internal error |
| `RateLimited` | 429 | Rate limit exceeded |
| `AccountLocked(msg)` | 403 | Brute-force lockout |
| `InvalidTotp` | 401 | Wrong TOTP code |
| `UserNotFound` | 404 | User lookup failed |
| `EmailExists` | 409 | Duplicate registration |
| `InvalidVerificationToken` | 400 | Bad email verification token |
| `OAuthError(msg)` | 400 | OAuth provider error |

### HTTP response

```rust
let response = err.to_response();
// AuthErrorResponse { status_code: 401, error_code: "INVALID_TOKEN", message: "..." }
```

`AuthError` implements `IntoResponse` for Axum via `web::axum::error`.

---

## Tests

All tests live in the `#[cfg(test)]` block inside [src/jwt.rs](../src/jwt.rs):

- `sign_and_verify` — round-trip with role checks
- `invalid_token_rejected` — garbage input returns `InvalidToken`
- `sign_and_verify_refresh` — refresh token round-trip
- `access_token_rejected_as_refresh` — type discrimination
- `exchange_returns_new_pair` — full refresh flow
- `wrong_secret_rejected` — cross-secret verification fails
