# rok-auth CLI Command Reference

Complete reference for the `rok-auth` binary. For an overview see [phase-09-cli-commands.md](phase-09-cli-commands.md).

---

## Global Flags

| Flag | Description |
|------|-------------|
| `--config <path>` | Path to `rok-auth.toml` (default: `./rok-auth.toml`) |
| `--json <payload>` | Pass input as a JSON string |
| `--output json\|table\|plain` | Output format (default: `table`) |
| `--quiet` | Suppress all output except errors |

---

## key

### `key generate`

Generate a new HMAC signing key.

```bash
rok-auth key generate [--bits 256|384|512] [--output <path>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--bits` | `256` | Key size in bits |
| `--output` | stdout | Write key to file |

### `key rotate`

Rotate the active key. The old key is retained for verification of existing tokens until they expire.

```bash
rok-auth key rotate [--keep-old <hours>]
```

### `key list`

List all configured keys and their status.

```bash
rok-auth key list
```

---

## token

### `token sign`

Sign a new JWT access token.

```bash
rok-auth token sign --sub <user-id> [--roles <role,...>] [--ttl <duration>] [--iss <issuer>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--sub` | required | Subject (user ID) |
| `--roles` | `""` | Comma-separated roles |
| `--ttl` | config value | Token lifetime (e.g. `2h`, `30m`) |
| `--iss` | config value | Issuer claim |

**Example:**
```bash
rok-auth token sign --sub user-123 --roles admin,user --ttl 2h
```

### `token verify`

Verify a JWT token. Exits `0` if valid, `1` if invalid or expired.

```bash
rok-auth token verify --token <jwt>
```

### `token decode`

Decode a JWT without verifying the signature (inspect claims only):

```bash
rok-auth token decode --token <jwt>
```

### `token refresh`

Exchange a refresh token for a new access + refresh token pair:

```bash
rok-auth token refresh --token <refresh-jwt>
```

---

## user

### `user create`

```bash
rok-auth user create --email <email> --password <password> [--roles <role,...>] [--name <name>]
```

### `user verify`

Mark a user's email as verified:

```bash
rok-auth user verify --email <email>
```

### `user reset-password`

Send a password reset email to the user:

```bash
rok-auth user reset-password --email <email>
```

### `user disable`

Disable a user account (all sessions are invalidated):

```bash
rok-auth user disable --id <user-id>
```

### `user list`

List users, optionally filtered by role:

```bash
rok-auth user list [--role <role>] [--limit <n>] [--offset <n>]
```

---

## session

### `session list`

List active sessions for a user:

```bash
rok-auth session list --user <user-id>
```

### `session revoke`

Revoke a single session:

```bash
rok-auth session revoke --session-id <id>
```

### `session revoke-all`

Revoke all sessions for a user (logout everywhere):

```bash
rok-auth session revoke-all --user <user-id>
```

---

## oauth

### `oauth list-providers`

List configured OAuth providers:

```bash
rok-auth oauth list-providers
```

### `oauth add`

Add an OAuth provider to the config:

```bash
rok-auth oauth add --provider google --client-id <id> --client-secret <secret> --redirect-uri <uri>
```

### `oauth remove`

Remove a configured provider:

```bash
rok-auth oauth remove --provider google
```

### `oauth link`

Link an OAuth identity to an existing user account:

```bash
rok-auth oauth link --user <user-id> --provider google --provider-user-id <id>
```

---

## audit

### `audit list`

List audit log events:

```bash
rok-auth audit list [--user <user-id>] [--days <n>] [--event <type>] [--failed-only]
```

---

## stats

### `stats auth`

Show authentication metrics:

```bash
rok-auth stats auth [--since <duration>]
```

Output includes: total logins, failed logins, tokens issued, active sessions, rate limit hits.

---

## health

### `health check`

Check the health of all components:

```bash
rok-auth health check [--component jwt|db|email|oauth]
```

Exit codes: `0` = healthy, `1` = degraded, `2` = unhealthy.
