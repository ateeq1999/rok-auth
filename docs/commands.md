# CLI Commands Specification

This document specifies the CLI commands for the `rok-cli` crate that interact with `rok-auth`.

## Command Conventions

### JSON Payload
All commands accept JSON payload via `--json-payload` flag for agent-friendly interaction.

```bash
rok-cli <command> --json '{
  "key": "value"
}'
```

### Output Formats
- `--output-format json` (default for piping)
- `--output-format table` (default for terminal)
- `--output-format yaml`

### Global Flags
| Flag | Description |
|------|-------------|
| `--json <json>` | JSON input |
| `--output-format <fmt>` | Output format (json/table/yaml) |
| `--config <path>` | Config file path |
| `--quiet` | Suppress output |
| `--verbose` | Verbose logging |

---

## Key Management

### `rok-cli key generate`

Generate a new signing key.

**JSON Payload:**
```json
{
  "algorithm": "HS256",
  "bits": 256
}
```

**Flags:**
- `--algorithm <algo>`: Signing algorithm (HS256, HS384, HS512, RS256, RS384, RS512)
- `--bits <n>`: Key size in bits (default: 256)
- `--output <path>`: Output file path (default: stdout)

**Example:**
```bash
rok-cli key generate --json '{"algorithm": "HS256"}' --output ./keys/jwt.key
```

---

### `rok-cli key rotate`

Rotate signing keys with zero-downtime support.

**JSON Payload:**
```json
{
  "current_key": "/path/to/current.key",
  "new_key": "/path/to/new.key"
}
```

**Flags:**
- `--current <path>`: Current key path
- `--new <path>`: New key path
- `--grace-period <duration>`: Old key validity period after rotation

**Example:**
```bash
rok-cli key rotate --current ./keys/jwt.key --new ./keys/jwt-new.key
```

---

### `rok-cli key list`

List all signing keys in the system.

**JSON Payload:**
```json
{
  "status": "active"
}
```

**Flags:**
- `--status <status>`: Filter by status (active, revoked, rotated)

---

## Token Operations

### `rok-cli token sign`

Sign a JWT token.

**JSON Payload:**
```json
{
  "sub": "user-123",
  "roles": ["admin", "user"],
  "custom_claims": {
    "email": "user@example.com"
  },
  "ttl": "1h"
}
```

**Flags:**
- `--sub <subject>`: Subject (user ID)
- `--roles <roles>`: Comma-separated roles
- `--ttl <duration>`: Token TTL (default: 1h)
- `--issuer <issuer>`: Token issuer

**Example:**
```bash
rok-cli token sign --json '{"sub": "user-123", "roles": ["admin"]}'
```

---

### `rok-cli token verify`

Verify a JWT token.

**JSON Payload:**
```json
{
  "token": "eyJ..."
}
```

**Flags:**
- `--token <token>`: Token to verify
- `--strict`: Strict verification (exp, nbf)

**Output:**
```json
{
  "valid": true,
  "claims": {
    "sub": "user-123",
    "roles": ["admin"],
    "exp": 1234567890
  }
}
```

---

### `rok-cli token decode`

Decode a JWT without verification.

**JSON Payload:**
```json
{
  "token": "eyJ..."
}
```

**Output:**
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "user-123",
    "roles": ["admin"]
  }
}
```

---

### `rok-cli token refresh`

Exchange a refresh token for new tokens.

**JSON Payload:**
```json
{
  "refresh_token": "eyJ..."
}
```

**Output:**
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 3600
}
```

---

## User Management

### `rok-cli user create`

Create a new user.

**JSON Payload:**
```json
{
  "email": "user@example.com",
  "password": "secure-password",
  "roles": ["user"],
  "email_verified": false
}
```

**Flags:**
- `--email <email>`: User email
- `--password <password>`: User password
- `--roles <roles>`: Comma-separated roles
- `--no-email-verification`: Skip email verification

**Output:**
```json
{
  "id": "user-uuid",
  "email": "user@example.com",
  "roles": ["user"],
  "created_at": "2024-01-01T00:00:00Z"
}
```

---

### `rok-cli user verify`

Verify a user's email.

**JSON Payload:**
```json
{
  "token": "verification-token"
}
```

**Flags:**
- `--token <token>`: Verification token

---

### `rok-cli user reset-password`

Initiate password reset for a user.

**JSON Payload:**
```json
{
  "email": "user@example.com"
}
```

**Flags:**
- `--email <email>`: User email
- `--send-email`: Send reset email (default: true)

---

### `rok-cli user confirm-reset`

Confirm password reset with token.

**JSON Payload:**
```json
{
  "token": "reset-token",
  "new_password": "new-secure-password"
}
```

---

### `rok-cli user disable`

Disable a user account.

**JSON Payload:**
```json
{
  "user_id": "user-uuid",
  "reason": "Policy violation"
}
```

**Flags:**
- `--user-id <id>`: User ID
- `--reason <reason>`: Disable reason

---

### `rok-cli user list`

List users with pagination.

**JSON Payload:**
```json
{
  "page": 1,
  "limit": 20,
  "filter": {
    "role": "admin",
    "verified": true
  }
}
```

**Flags:**
- `--page <n>`: Page number
- `--limit <n>`: Items per page
- `--role <role>`: Filter by role

**Output:**
```json
{
  "users": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "pages": 5
  }
}
```

---

### `rok-cli user get`

Get user details.

**JSON Payload:**
```json
{
  "user_id": "user-uuid"
}
```

**Flags:**
- `--user-id <id>`: User ID

---

## Session Management

### `rok-cli session list`

List active sessions for a user.

**JSON Payload:**
```json
{
  "user_id": "user-uuid",
  "page": 1,
  "limit": 20
}
```

**Flags:**
- `--user-id <id>`: User ID

---

### `rok-cli session revoke`

Revoke a specific session.

**JSON Payload:**
```json
{
  "session_id": "session-uuid"
}
```

**Flags:**
- `--session-id <id>`: Session ID

---

### `rok-cli session revoke-all`

Revoke all sessions for a user.

**JSON Payload:**
```json
{
  "user_id": "user-uuid"
}
```

**Flags:**
- `--user-id <id>`: User ID
- `--except-current`: Keep current session

---

## OAuth Management

### `rok-cli oauth list-providers`

List configured OAuth providers.

**Output:**
```json
{
  "providers": [
    {
      "name": "google",
      "enabled": true,
      "scopes": ["openid", "email", "profile"]
    }
  ]
}
```

---

### `rok-cli oauth add`

Add an OAuth provider configuration.

**JSON Payload:**
```json
{
  "name": "google",
  "client_id": "xxx",
  "client_secret": "yyy",
  "auth_url": "https://accounts.google.com/o/oauth2/v2/auth",
  "token_url": "https://oauth2.googleapis.com/token",
  "scopes": ["openid", "email", "profile"]
}
```

---

### `rok-cli oauth remove`

Remove an OAuth provider.

**JSON Payload:**
```json
{
  "name": "google"
}
```

**Flags:**
- `--provider <name>`: Provider name

---

### `rok-cli oauth link`

Link an OAuth account to a user.

**JSON Payload:**
```json
{
  "user_id": "user-uuid",
  "provider": "google",
  "provider_user_id": "google-user-id"
}
```

---

## Audit & Monitoring

### `rok-cli audit list`

List audit log entries.

**JSON Payload:**
```json
{
  "user_id": "user-uuid",
  "from": "2024-01-01T00:00:00Z",
  "to": "2024-01-31T23:59:59Z",
  "event_type": "login",
  "page": 1,
  "limit": 50
}
```

**Flags:**
- `--user-id <id>`: Filter by user
- `--from <date>`: Start date
- `--to <date>`: End date
- `--event <type>`: Event type (login, logout, token_refresh, etc.)

---

### `rok-cli stats auth`

Get authentication statistics.

**JSON Payload:**
```json
{
  "period": "24h",
  "metrics": ["login_attempts", "failures", "token_refreshes"]
}
```

**Flags:**
- `--period <duration>`: Time period (default: 24h)
- `--granularity <granularity>`: Data granularity (1m, 5m, 1h, 1d)

---

### `rok-cli health check`

Check system health.

**Output:**
```json
{
  "status": "healthy",
  "components": {
    "database": "ok",
    "redis": "ok",
    "smtp": "ok"
  },
  "checks": {
    "token_service": true,
    "session_service": true
  }
}
```

---

## Error Responses

All commands return structured error responses:

```json
{
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Token has expired",
    "details": {}
  }
}
```

Exit codes:
- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `3`: Authentication error
- `4`: Authorization error
- `5`: Not found
- `6`: Rate limited
