# Phase 8: Rate Limiting & Security Hardening

Rate limiting, brute-force protection, IP reputation, CORS, CSRF, security headers, device tracking, health monitoring, and security webhooks. Implemented in [src/security/](../src/security/).

---

## RateLimiter

Defined in [src/security/rate_limiter.rs](../src/security/rate_limiter.rs). Combines a **token bucket** (for burst control) with a **sliding window** (for sustained rate limiting).

### Configuration

```rust
pub struct RateLimitConfig {
    pub requests_per_window: u32,  // max requests in the window
    pub window_secs: u64,          // window length in seconds
    pub burst_size: u32,           // burst capacity above the sustained rate
}
```

Default: 100 req/60s, burst of 10.

### Basic usage

```rust
use rok_auth::{RateLimiter, RateLimitConfig, RateLimitResult};

let limiter = RateLimiter::new(RateLimitConfig {
    requests_per_window: 60,
    window_secs: 60,
    burst_size: 5,
});

match limiter.check_ip("192.168.1.1").await {
    RateLimitResult::Allowed => { /* proceed */ }
    RateLimitResult::RateLimited { retry_after_secs } => {
        // return 429 Too Many Requests with Retry-After header
    }
}
```

### Keyed checks

```rust
limiter.check_user("user-123").await;           // keyed by user ID
limiter.check_ip("192.168.1.1").await;          // keyed by IP address
limiter.check_endpoint("1.2.3.4", "/login").await; // keyed by IP + endpoint
limiter.check("custom-key").await;              // arbitrary key
```

### MultiRateLimiter

Different limits per endpoint:

```rust
use rok_auth::MultiRateLimiter;

let mut multi = MultiRateLimiter::new();
multi.add_limiter("login", RateLimitConfig { requests_per_window: 5, window_secs: 300, burst_size: 2 });
multi.add_limiter("api",   RateLimitConfig { requests_per_window: 1000, window_secs: 60, burst_size: 50 });

let result = multi.check("login", "ip:1.2.3.4").await;
```

### Cleanup

The limiter holds state in memory. Call `cleanup` periodically to cap memory usage:

```rust
limiter.cleanup(10_000).await; // keep at most 10,000 entries
```

---

## BruteForceDetector

Defined in [src/security/detection.rs](../src/security/detection.rs). Tracks failed login attempts per identifier and applies temporary lockouts.

```rust
use rok_auth::BruteForceDetector;
use std::time::Duration;

// Default: 5 attempts, 15-minute lockout, 5-minute detection window
let detector = BruteForceDetector::default();

// Custom: 3 attempts, 1-hour lockout, 10-minute window
let detector = BruteForceDetector::new(3, Duration::from_secs(3600), Duration::from_secs(600));

// Record a failed attempt
match detector.record_failed_attempt("alice@example.com").await {
    AttemptResult::Allowed { attempts_remaining } => { /* show warning */ }
    AttemptResult::Locked { remaining_secs } => {
        return Err(AuthError::AccountLocked(format!("locked for {}s", remaining_secs)));
    }
}

// Clear on successful login
detector.record_successful_attempt("alice@example.com").await;

// Manual unlock (admin action)
detector.unlock("alice@example.com").await;
```

---

## IpReputationChecker

Tracks IP reputation scores. A score of ≤ -100 blocks the IP.

```rust
use rok_auth::IpReputationChecker;

let checker = IpReputationChecker::new();

// Record bad behavior (negative delta)
checker.report_bad_ip("1.2.3.4", -50, "brute_force").await;
checker.report_bad_ip("1.2.3.4", -60, "rate_limit_exceeded").await;

let result = checker.check_ip("1.2.3.4").await;
// IpReputationResult { blocked: true, suspicious: true, score: -110, flags: [...] }

// Whitelist (e.g., for internal IPs)
checker.whitelist_ip("10.0.0.1").await;

// Cleanup stale entries
checker.cleanup_stale(Duration::from_secs(86400)).await;
```

---

## CredentialStuffingDetector

Detects credential stuffing attacks by watching for too many distinct users or reused passwords from a single IP.

```rust
use rok_auth::security::detection::CredentialStuffingDetector;
use rok_auth::{StuffingResult, StuffingReason};

// Default: 5 users/IP/hour, 3 reused passwords
let detector = CredentialStuffingDetector::default();

match detector.check_login("1.2.3.4", "user@example.com", &password_hash).await {
    StuffingResult::Allowed => { /* proceed */ }
    StuffingResult::Suspicious { reason, score } => match reason {
        StuffingReason::TooManyUsersPerIp => { /* block */ }
        StuffingReason::PasswordReuse => { /* block */ }
    }
}
```

---

## SecurityHeaders

Defined in [src/security/headers.rs](../src/security/headers.rs). Configures HTTP security response headers.

```rust
use rok_auth::SecurityHeaders;

// Sensible defaults (HSTS, CSP, X-Frame-Options: DENY, etc.)
let headers = SecurityHeaders::new();

// Maximum security (stricter CSP, HSTS with preload)
let headers = SecurityHeaders::new().strict();

// Relaxed (useful for development)
let headers = SecurityHeaders::new().permissive();

// Custom HSTS and CSP
let headers = SecurityHeaders::new()
    .with_hsts(63_072_000, true)  // 2-year HSTS with subdomains
    .with_csp("default-src 'self'; img-src *");
```

Default headers applied:
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- `Content-Security-Policy: default-src 'self'; ...`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: geolocation=(), microphone=(), camera=()`

---

## CorsConfig

Defined in [src/security/cors.rs](../src/security/cors.rs).

```rust
use rok_auth::CorsConfig;

let cors = CorsConfig::permissive();        // allow all origins (dev only)
let cors = CorsConfig::api_defaults();      // standard API CORS policy
let cors = CorsConfig::same_origin_only();  // no cross-origin requests
```

---

## CsrfProtection

Defined in [src/security/csrf.rs](../src/security/csrf.rs). Generates and validates CSRF tokens for SPA / cookie-based auth flows.

```rust
use rok_auth::CsrfProtection;

let csrf = CsrfProtection::new("csrf-secret".to_string());

// Generate on session start
let token = csrf.generate("session-id-123");

// Validate on state-mutating requests
let valid = csrf.validate("session-id-123", &token);
```

---

## StepUpAuth

Defined in [src/security/step_up.rs](../src/security/step_up.rs). Enforces re-authentication for sensitive operations by checking token freshness.

```rust
use rok_auth::StepUpAuth;

let step_up = StepUpAuth::new(); // default freshness window: 5 minutes

if step_up.requires_reauth(&claims) {
    return Err(AuthError::Forbidden("step-up authentication required".to_string()));
}
```

---

## DeviceManager

Defined in [src/security/device.rs](../src/security/device.rs). Tracks per-user devices/sessions.

```rust
use rok_auth::{DeviceManager, Device, DeviceType};

let manager = DeviceManager::new();

// Register a device on login
let device = Device::new("user-123".to_string(), DeviceType::Web)
    .with_name("Chrome on Windows");
let device_id = manager.register(device).await;

// List all devices for a user
let devices = manager.get_user_devices("user-123").await;

// Revoke a single device (logout from one session)
manager.revoke_device(&device_id).await;

// Revoke all devices (logout everywhere)
manager.revoke_all_user_devices("user-123").await;
```

---

## HealthChecker & MetricsCollector

Defined in [src/security/health.rs](../src/security/health.rs).

```rust
use rok_auth::HealthChecker;

let checker = HealthChecker::new();
let status = checker.check().await;
// HealthStatus { state: HealthState::Healthy, checks: [...] }
```

```rust
use rok_auth::MetricsCollector;

let metrics = MetricsCollector::new();
metrics.record_login_success().await;
metrics.record_login_failure().await;
metrics.record_token_issued().await;

let snapshot = metrics.snapshot().await;
// AuthMetrics { login_successes, login_failures, tokens_issued, ... }
```

---

## SecurityWebhook

Defined in [src/security/webhook.rs](../src/security/webhook.rs). Sends signed HTTP POST notifications to an external endpoint on security events.

```rust
use rok_auth::SecurityWebhook;

let webhook = SecurityWebhook::new(
    "https://your-app.com/webhooks/security".to_string(),
    "webhook-signing-secret".to_string(),
);

// Send a security event (signs payload with HMAC-SHA256)
webhook.send(&event).await?;
```

Payloads are signed with `X-Signature: sha256=<hex>` so the receiver can verify authenticity.
