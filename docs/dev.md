# Developer Notes - Security Enhancements

This document outlines potential security enhancements for rok-auth, inspired by Laravel Sanctum and JWT best practices.

## Current Implementation Status

All 9 phases are complete with 79 tests passing.

## Recommended Enhancements

### 1. Token Management Enhancements

#### 1.1 Token Abilities/Scopes
Inspired by Laravel Sanctum's token abilities:

```rust
// New: TokenAbilities enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenAbility {
    Read,
    Write,
    Delete,
    Manage,
    Custom(String),
}

impl TokenAbility {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "read" => TokenAbility::Read,
            "write" => TokenAbility::Write,
            "delete" => TokenAbility::Delete,
            "manage" => TokenAbility::Manage,
            other => TokenAbility::Custom(other.to_string()),
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            TokenAbility::Read => "read",
            TokenAbility::Write => "write",
            TokenAbility::Delete => "delete",
            TokenAbility::Manage => "manage",
            TokenAbility::Custom(s) => s,
        }
    }
}

// Token with abilities
#[derive(Debug, Clone)]
pub struct TokenWithAbilities {
    pub token: String,
    pub abilities: Vec<TokenAbility>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub name: Option<String>,  // e.g., "iPhone 12", "Work Laptop"
}

impl TokenWithAbilities {
    pub fn new(token: String, abilities: Vec<TokenAbility>) -> Self {
        Self {
            token,
            abilities,
            expires_at: None,
            created_at: Utc::now(),
            name: None,
        }
    }
    
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }
    
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn can(&self, ability: &TokenAbility) -> bool {
        self.abilities.iter().any(|a| a == ability)
    }
    
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }
}
```

#### 1.2 Token Revocation (Blacklist)
JWT token revocation strategy:

```rust
// Token blacklist for immediate invalidation
pub struct TokenBlacklist {
    store: Arc<tokio::sync::RwLock<HashMap<String, Instant>>>,
    cleanup_interval: Duration,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(3600),
        }
    }
    
    // Add token ID (jti) to blacklist until expiry
    pub async fn revoke(&self, jti: &str, expires_at: DateTime<Utc>) {
        let mut blacklist = self.store.write().await;
        blacklist.insert(jti.to_string(), expires_at);
    }
    
    // Check if token is blacklisted
    pub async fn is_revoked(&self, jti: &str) -> bool {
        let blacklist = self.store.read().await;
        if let Some(expires_at) = blacklist.get(jti) {
            Utc::now() < *expires_at
        } else {
            false
        }
    }
    
    // Cleanup expired entries
    pub async fn cleanup(&self) {
        let mut blacklist = self.store.write().await;
        let now = Utc::now();
        blacklist.retain(|_, exp| now < *exp);
    }
}
```

### 2. Device/Session Management

#### 2.1 Device Tracking
Inspired by Laravel Sanctum's mobile authentication:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub id: String,
    pub user_id: String,
    pub name: Option<String>,  // "Chrome on Windows", "iPhone 12"
    pub device_type: DeviceType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    Web,
    Mobile,
    Desktop,
    Api,
    Unknown,
}

impl Device {
    pub fn new(user_id: String, device_type: DeviceType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            name: None,
            device_type,
            ip_address: None,
            user_agent: None,
            last_active: Utc::now(),
            created_at: Utc::now(),
            expires_at: None,
        }
    }
    
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }
}

// Device manager for tracking user sessions
pub struct DeviceManager {
    devices: Arc<tokio::sync::RwLock<HashMap<String, Device>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, device: Device) -> String {
        let mut devices = self.devices.write().await;
        devices.insert(device.id.clone(), device);
        device.id
    }
    
    pub async fn get_user_devices(&self, user_id: &str) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter(|d| d.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub async fn revoke_device(&self, device_id: &str) -> bool {
        let mut devices = self.devices.write().await;
        devices.remove(device_id).is_some()
    }
    
    pub async fn revoke_all_user_devices(&self, user_id: &str) {
        let mut devices = self.devices.write().await;
        devices.retain(|_, d| d.user_id != user_id);
    }
    
    pub async fn cleanup_expired(&self) {
        let mut devices = self.devices.write().await;
        let now = Utc::now();
        devices.retain(|_, d| {
            d.expires_at.map_or(true, |exp| now < exp)
        });
    }
}
```

### 3. SPA Authentication Support

#### 3.1 CSRF Protection
Inspired by Laravel Sanctum's CSRF cookie flow:

```rust
// CSRF token generation and validation
pub struct CsrfProtection {
    secret: Arc<String>,
    store: Arc<tokio::sync::RwLock<HashMap<String, Instant>>>,
}

impl CsrfProtection {
    pub fn new(secret: String) -> Self {
        Self {
            secret: Arc::new(secret),
            store: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    // Generate a new CSRF token
    pub fn generate(&self, session_id: &str) -> String {
        let token = format!(
            "{}.{}",
            hex::encode(rand::random::<[u8; 16]>()),
            hex::encode(rand::random::<[u8; 16]>())
        );
        
        // Store token with expiry (typically 2 hours)
        let expiry = Utc::now() + Duration::from_secs(7200);
        
        // In real implementation, this would be stored in session/cache
        // For now, we use an in-memory store
        let mut store = self.store.blocking_write();
        store.insert(format!("{}:{}", session_id, &token), expiry);
        
        token
    }
    
    // Validate a CSRF token
    pub fn validate(&self, session_id: &str, token: &str) -> bool {
        let mut store = self.store.write().now_or().unwrap();
        let key = format!("{}:{}", session_id, token);
        
        if let Some(expires_at) = store.get(&key) {
            if Utc::now() < *expires_at {
                return true;
            }
        }
        false
    }
}
```

### 4. Advanced JWT Security

#### 4.1 Token Freshness (Fresh Token Requirement)
Inspired by JWT best practices for sensitive operations:

```rust
// Add fresh claim to tokens
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FreshClaims {
    pub sub: String,
    pub roles: Vec<String>,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub fresh: bool,  // Token was recently validated with password/TOTP
    pub sid: Option<String>,  // Session ID
    pub jti: String,  // JWT ID for revocation
}

impl FreshClaims {
    pub fn new(sub: String, roles: Vec<String>, issuer: String) -> Self {
        Self {
            sub,
            roles,
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + Duration::from_secs(900)).timestamp(),  // 15 min default
            iss: issuer,
            fresh: true,
            sid: None,
            jti: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    pub fn mark_not_fresh(mut self) -> Self {
        self.fresh = false;
        self
    }
    
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.sid = Some(session_id.into());
        self
    }
}
```

#### 4.2 Algorithm Header Validation
Prevent algorithm confusion attacks:

```rust
// Strict algorithm validation
pub enum Algorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
}

impl Algorithm {
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::HS256 => "HS256",
            Algorithm::HS384 => "HS384",
            Algorithm::HS512 => "HS512",
            Algorithm::RS256 => "RS256",
            Algorithm::RS384 => "RS384",
            Algorithm::RS512 => "RS512",
        }
    }
}

// Strict token validation
pub struct StrictValidator {
    allowed_algorithms: Vec<Algorithm>,
    require_claims: Vec<&'static str>,
    blacklist: TokenBlacklist,
}

impl StrictValidator {
    pub fn new() -> Self {
        Self {
            // Only allow HMAC algorithms by default (prevents alg:none attacks)
            allowed_algorithms: vec![Algorithm::HS256, Algorithm::HS384, Algorithm::HS512],
            require_claims: vec!["sub", "exp", "iat"],
            blacklist: TokenBlacklist::new(),
        }
    }
    
    pub fn allow_rsa(mut self) -> Self {
        self.allowed_algorithms.push(Algorithm::RS256);
        self.allowed_algorithms.push(Algorithm::RS384);
        self.allowed_algorithms.push(Algorithm::RS512);
        self
    }
    
    pub fn validate(&self, token: &str, secret: &str) -> Result<Claims, AuthError> {
        // Parse header and validate algorithm
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidToken);
        }
        
        let header: serde_json::Value = serde_json::from_str(
            &base64_decode(parts[0])
        ).map_err(|_| AuthError::InvalidToken)?;
        
        let alg = header["alg"]
            .as_str()
            .ok_or(AuthError::InvalidToken)?;
        
        // Strict algorithm check
        if !self.allowed_algorithms.iter().any(|a| a.as_str() == alg) {
            return Err(AuthError::Internal("Invalid algorithm".to_string()));
        }
        
        // Reject none algorithm
        if alg.eq_ignore_ascii_case("none") {
            return Err(AuthError::Internal("Algorithm 'none' not allowed".to_string()));
        }
        
        // Verify and decode token
        // ... rest of validation
        Ok(Claims::default())
    }
}
```

### 5. Multi-Factor Authentication Enhancements

#### 5.1 Step-up Authentication
Require fresh authentication for sensitive operations:

```rust
// Step-up authentication guard
pub struct StepUpAuth {
    freshness_duration: Duration,
}

impl StepUpAuth {
    pub fn new() -> Self {
        Self {
            freshness_duration: Duration::from_secs(300),  // 5 minutes
        }
    }
    
    pub fn requires_reauth(&self, claims: &Claims) -> bool {
        // Check if token was issued within freshness window
        let issued_at = DateTime::from_timestamp(claims.iat, 0)
            .unwrap_or_else(Utc::now);
        Utc::now() - issued_at > self.freshness_duration
    }
}
```

### 6. Audit & Logging Enhancements

#### 6.1 Structured Audit Events
Inspired by security best practices:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityAuditEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub details: serde_json::Value,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum SecurityEventType {
    Login,
    Logout,
    TokenRefresh,
    PasswordChange,
    PasswordReset,
    MfaEnabled,
    MfaDisabled,
    MfaChallenge,
    TokenRevoked,
    DeviceRegistered,
    DeviceRevoked,
    SuspiciousActivity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
```

### 7. Webhook Notifications

#### 7.1 Security Event Webhooks
Notify external services of security events:

```rust
pub struct SecurityWebhook {
    url: String,
    secret: String,
    events: Vec<SecurityEventType>,
}

impl SecurityWebhook {
    pub fn new(url: String, secret: String) -> Self {
        Self {
            url,
            secret,
            events: vec![
                SecurityEventType::PasswordChange,
                SecurityEventType::MfaEnabled,
                SecurityEventType::DeviceRegistered,
                SecurityEventType::SuspiciousActivity,
            ],
        }
    }
    
    pub async fn send(&self, event: &SecurityAuditEvent) -> Result<(), AuthError> {
        if !self.events.contains(&event.event_type) {
            return Ok(());
        }
        
        let payload = serde_json::to_vec(event)
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        
        let signature = self.compute_signature(&payload);
        
        // Send HTTP POST request with signature
        // ... implementation using reqwest
        
        Ok(())
    }
    
    fn compute_signature(&self, payload: &[u8]) -> String {
        use hmac_sha256::HMAC;
        hex::encode(HMAC::mac(payload, self.secret.as_bytes()))
    }
}
```

## Implementation Priority

| Feature | Priority | Complexity | Database Required |
|---------|----------|-------------|-------------------|
| Token Abilities | High | Low | No |
| Token Blacklist | High | Medium | Yes |
| Device Tracking | Medium | Medium | Yes |
| Strict Algorithm Validation | High | Low | No |
| CSRF Protection | Medium | Medium | No |
| Step-up Auth | Medium | Low | No |
| Structured Audit Events | Low | Low | Yes |
| Security Webhooks | Low | Medium | No |

## Notes

- Database-related features are marked as requiring a database layer
- All security enhancements should be optional and configurable
- Default configurations should be secure by default
- Consider adding feature flags for gradual rollout

## References

- [Laravel Sanctum Documentation](https://laravel.com/docs/master/sanctum)
- [JWT.io Security Best Practices](https://jwt.io/introduction)
- [OWASP API Security Guidelines](https://owasp.org/www-project-api-security/)
- [IETF JWT Best Current Practices (RFC 8725)](https://datatracker.ietf.org/doc/rfc8725/)
