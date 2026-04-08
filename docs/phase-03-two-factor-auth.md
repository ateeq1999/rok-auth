# Phase 3: Two-Factor Authentication (TOTP)

RFC 6238 Time-based One-Time Passwords with backup codes, implemented in [src/services/totp.rs](../src/services/totp.rs).

---

## TotpConfig

```rust
pub struct TotpConfig {
    pub digits: u32,          // Code length (default: 6)
    pub period_secs: u64,     // Time step (default: 30)
    pub algorithm: TotpAlgorithm, // SHA-1 | SHA-256 | SHA-512 (default: SHA-1)
}
```

`TotpConfig::default()` matches the Google Authenticator / Authy standard: 6 digits, 30-second steps, SHA-1.

---

## TotpService

### Construction

```rust
use rok_auth::services::totp::{TotpService, TotpConfig};

let service = TotpService::new(TotpConfig::default());
```

### Generate a secret

Call once during user 2FA enrollment. Store the result encrypted in your database.

```rust
let secret = service.generate_secret();
// Base32 string, e.g. "JBSWY3DPEHPK3PXP"
```

### Generate a provisioning URI

Returns an `otpauth://totp/...` URI. Render it as a QR code for users to scan.

```rust
let uri = service.provisioning_uri(
    &secret,
    "alice@example.com", // account label
    "MyApp",             // issuer
);
// "otpauth://totp/MyApp:alice%40example.com?secret=...&issuer=MyApp&digits=6&period=30"
```

### Verify a user-supplied code

```rust
// tolerance = number of adjacent time steps to accept (1 = ±30s clock drift)
let valid = service.verify_code(&secret, "482913", 1)?;
if !valid {
    return Err(AuthError::InvalidTotp);
}
```

Uses constant-time comparison to prevent timing attacks.

### Generate a code (testing / seeding)

```rust
let code = service.generate_code(&secret)?;
println!("current TOTP: {}", code.as_str()); // "482913"
```

---

## BackupCodes

One-time recovery codes for users who lose their authenticator device.

```rust
use rok_auth::services::totp::BackupCodes;

// Generate 10 codes (give raw codes to user; store hashed copies in DB)
let mut codes = BackupCodes::generate(10);
println!("{} remaining", codes.remaining()); // 10

// Verify and consume a code
let valid = codes.verify("a1b2c3d4e5f6g7h8"); // true on first use
let again = codes.verify("a1b2c3d4e5f6g7h8"); // false — already consumed
println!("{} remaining", codes.remaining()); // 9
```

Each code is a 16-char hex string (8 random bytes). Verified with constant-time comparison.

---

## Enrollment Flow

```
1. User enables 2FA in account settings
2. generate_secret()       → store encrypted in DB, mark status: unconfirmed
3. provisioning_uri()      → render as QR code
4. User scans with authenticator app
5. User submits the displayed 6-digit code
6. verify_code()           → if valid, set status: confirmed in DB
7. BackupCodes::generate() → show raw codes once; store hashed copies in DB
```

---

## Security Notes

- `tolerance: 1` permits ±30 seconds of clock drift — sufficient for most deployments
- Store TOTP secrets **encrypted** at rest (AES-256-GCM or equivalent)
- Store backup code **hashes**, never raw backup codes
- Invalidate a backup code immediately on first use
