# Phase 7: API Polish & Developer Experience

Builder patterns, utility functions, and procedural macros that make rok-auth ergonomic to use. Implemented in [src/builders.rs](../src/builders.rs), [src/utils.rs](../src/utils.rs), and [rok-auth-macros/src/lib.rs](../rok-auth-macros/src/lib.rs).

---

## AuthConfigBuilder

A fluent builder for `AuthConfig` that prevents invalid configurations at compile time.

```rust
use rok_auth::{Auth, AuthConfigBuilder};

let auth = Auth::new(
    AuthConfigBuilder::new()
        .secret("my-secret-key")       // required
        .token_ttl_hours(2)            // access token: 2 hours
        .refresh_ttl_days(14)          // refresh token: 14 days
        .issuer("my-app")              // optional `iss` claim
        .build()
        .unwrap()
);
```

### TTL methods

| Method | Effect |
|--------|--------|
| `.token_ttl(Duration)` | Set TTL from a `Duration` directly |
| `.token_ttl_secs(u64)` | Set TTL in seconds |
| `.token_ttl_minutes(u64)` | Set TTL in minutes |
| `.token_ttl_hours(u64)` | Set TTL in hours |
| `.refresh_ttl(Duration)` | Set refresh TTL from a `Duration` |
| `.refresh_ttl_days(u64)` | Set refresh TTL in days |

### Errors

`build()` returns `Err(AuthConfigBuilderError)` if:
- `secret` was never called → `MissingSecret`
- `secret("")` was called → `EmptySecret`

`AuthConfigBuilderError` converts to `AuthError::Internal`.

---

## Utility Functions

Defined in [src/utils.rs](../src/utils.rs).

### auth_from_secret

Create an `Auth` instance from a single secret string:

```rust
use rok_auth::utils::auth_from_secret;

let auth = auth_from_secret("my-secret")?;
```

### auth_with_defaults

Create an `Auth` instance with a freshly generated random secret. Useful for development and testing:

```rust
use rok_auth::utils::auth_with_defaults;

let auth = auth_with_defaults()?;
```

### random_secret

Generate a cryptographically random 256-bit secret (64-char hex string):

```rust
use rok_auth::utils::random_secret;

let secret = random_secret(); // "a3f8b1c2d4..."
assert_eq!(secret.len(), 64);
```

Uses `rand::thread_rng().fill_bytes()` — suitable for production secret generation.

### parse_duration

Parse a human-readable duration string into `std::time::Duration`:

```rust
use rok_auth::utils::parse_duration;

parse_duration("60s")  // 60 seconds
parse_duration("5m")   // 300 seconds
parse_duration("2h")   // 7200 seconds
parse_duration("1d")   // 86400 seconds
parse_duration("1w")   // 604800 seconds
parse_duration("3600") // plain number treated as seconds
```

Returns `Err(ParseDurationError::InvalidNumber)` on unrecognized input.

### format_duration

Format a `Duration` to a compact human-readable string:

```rust
use rok_auth::utils::format_duration;
use std::time::Duration;

format_duration(Duration::from_secs(60))     // "1m"
format_duration(Duration::from_secs(3600))   // "1h"
format_duration(Duration::from_secs(86400))  // "1d"
format_duration(Duration::from_secs(604800)) // "1w"
```

### OptExt

Ergonomic conversion from `Option<T>` to `Result<T, AuthError>`:

```rust
use rok_auth::utils::OptExt;

let user = db.find_user(id).ok_or_auth_error()?;
// Returns Err(AuthError::Internal("unexpected None")) if None
```

---

## Procedural Macros

Defined in [rok-auth-macros/src/lib.rs](../rok-auth-macros/src/lib.rs).

### `#[require_role]`

Injects a role check at the top of an async Axum handler. Returns 403 if the `Claims` extractor is missing the required role.

```rust
use rok_auth_macros::require_role;
use rok_auth::Claims;

#[require_role("admin")]
async fn delete_user(claims: Claims) -> impl IntoResponse {
    // only reached if claims.has_role("admin")
    StatusCode::OK
}
```

### `#[require_any_role]`

Returns 403 if the user has none of the listed roles:

```rust
use rok_auth_macros::require_any_role;

#[require_any_role("editor", "admin")]
async fn edit_post(claims: Claims) -> impl IntoResponse {
    StatusCode::OK
}
```

The macros expand to role checks that run before the handler body, keeping handler code clean.
