# Phase 2: User Authentication Flows

Password hashing with Argon2id and cryptographically secure session tokens.

---

## Password Hashing

Defined in [src/password/hash.rs](../src/password/hash.rs). Uses **Argon2id** with a random salt per hash.

### Sync API

```rust
use rok_auth::password::{hash, verify};

let hash_str = hash("correct-horse-battery-staple")?;

let ok = verify("correct-horse-battery-staple", &hash_str)?;
assert!(ok);

let bad = verify("wrong-password", &hash_str)?;
assert!(!bad);
```

Each call to `hash` produces a different output — the salt is embedded in the PHC string.

### Async API

Argon2id is CPU-intensive. Use the non-blocking wrappers in async contexts:

```rust
use rok_auth::password::{hash_async, verify_async};

let hash_str = hash_async("my-password".to_string()).await?;
let ok = verify_async("my-password".to_string(), hash_str).await?;
```

Both use `tokio::task::spawn_blocking` so they never block the async executor.

### Public re-exports

`rok_auth::hash` and `rok_auth::verify` are re-exported at the crate root for convenience.

### Errors

Returns `Err(AuthError::HashError(msg))` on failure. Only happens on system entropy exhaustion — extremely rare.

---

## SessionToken

Defined in [src/session/token.rs](../src/session/token.rs). A 256-bit opaque token for session cookies or database-backed sessions.

```rust
use rok_auth::SessionToken;

// Generate a new random token (32 bytes -> 64-char hex string)
let token = SessionToken::generate();
println!("{}", token); // "a3f8b1..."

// Wrap an existing value loaded from storage
let token = SessionToken::wrap("existing-token-value");

// Access raw string
let s: &str = token.as_str();
```

`SessionToken` implements `Display`, `PartialEq`, `Eq`, and `Hash` — usable as a map key.

---

## UserProvider Trait

Defined in [src/providers/trait_.rs](../src/providers/trait_.rs). An async trait your application implements to connect rok-auth to your user store. rok-auth does not ship a built-in persistence layer.

```rust
use rok_auth::UserProvider;

struct MyDb { /* pool */ }

impl UserProvider for MyDb {
    type User = MyUser;

    async fn find_by_id(&self, id: &str) -> Option<Self::User> { todo!() }
    async fn find_by_email(&self, email: &str) -> Option<Self::User> { todo!() }
}
```

---

## Typical Login Flow

```rust
use rok_auth::{Auth, AuthError, Claims};
use rok_auth::password::verify_async;

async fn login(
    auth: &Auth,
    password_input: &str,
    stored_hash: &str,
    user_id: &str,
    roles: Vec<String>,
) -> Result<(String, String), AuthError> {
    // 1. Verify password (non-blocking)
    let ok = verify_async(password_input.to_string(), stored_hash.to_string()).await?;
    if !ok {
        return Err(AuthError::InvalidCredentials);
    }

    // 2. Issue access + refresh tokens
    let access = auth.sign(&Claims::new(user_id, roles))?;
    let refresh = auth.sign_refresh(user_id)?;

    Ok((access, refresh))
}
```
