# Phase 4: OAuth Integration

OAuth 2.0 social login support implemented in [src/services/oauth.rs](../src/services/oauth.rs).

---

## Core Types

### OAuthConfig

Generic OAuth 2.0 provider config:

```rust
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}
```

### OAuthTokens

Returned after a successful token exchange:

```rust
pub struct OAuthTokens {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub id_token: Option<String>,  // OpenID Connect
}
```

---

## Built-in Providers

rok-auth ships pre-configured providers for Google, GitHub, and Discord. Each provider wraps `OAuthConfig` with the correct auth/token URLs and default scopes.

### Google

```rust
use rok_auth::services::oauth::GoogleProvider;

let google = GoogleProvider::new(
    "client_id".to_string(),
    "client_secret".to_string(),
    "https://example.com/auth/callback/google".to_string(),
);
```

Default scopes: `openid email profile`

### GitHub

```rust
use rok_auth::services::oauth::GitHubProvider;

let github = GitHubProvider::new(
    "client_id".to_string(),
    "client_secret".to_string(),
    "https://example.com/auth/callback/github".to_string(),
);
```

Default scopes: `user:email`

### Discord

```rust
use rok_auth::services::oauth::DiscordProvider;

let discord = DiscordProvider::new(
    "client_id".to_string(),
    "client_secret".to_string(),
    "https://example.com/auth/callback/discord".to_string(),
);
```

Default scopes: `identify email`

---

## OAuthService

Wraps a provider and exposes the authorization URL and token exchange:

```rust
use rok_auth::services::oauth::OAuthService;

let service = OAuthService::new(google);

// 1. Redirect the user to this URL
let auth_url = service.authorization_url()?;

// 2. After the user is redirected back, exchange the code for tokens
let tokens = service.exchange_code("code-from-callback").await?;

// 3. Fetch the user's profile
let profile = service.fetch_user_info(&tokens.access_token).await?;
// OAuthUserInfo { id, email, name, avatar_url, ... }
```

---

## Custom Provider

Implement the `OAuthProvider` trait for any OAuth 2.0 compliant service:

```rust
use rok_auth::services::oauth::OAuthProvider;

struct MyProvider { config: OAuthConfig }

impl OAuthProvider for MyProvider {
    fn config(&self) -> &OAuthConfig { &self.config }
    fn provider_name(&self) -> &str { "myprovider" }
    // override fetch_user_info if the provider uses a non-standard endpoint
}
```

---

## Typical OAuth Flow

```
1. User clicks "Login with Google"
2. authorization_url() → redirect user to Google
3. Google redirects back to redirect_uri with ?code=...&state=...
4. exchange_code(code) → obtain OAuthTokens
5. fetch_user_info(access_token) → obtain email, name, avatar
6. Look up or create user in your DB by provider + provider_user_id
7. auth.sign(&Claims::new(user_id, roles)) → issue your own JWT
```

Always validate the `state` parameter (CSRF protection for the OAuth flow). Generate a random `state` before redirecting and verify it matches on callback.
