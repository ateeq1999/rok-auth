# Phase 9: CLI Commands

The `rok-auth` binary provides a developer CLI for key management, token operations, and diagnostics. A full `rok-cli` crate (standalone tool) is planned but deferred.

See [commands.md](commands.md) for the complete command reference.

---

## Running the CLI

```bash
cargo run --bin rok-auth -- <command> [options]
```

Or after building:

```bash
./target/debug/rok-auth <command> [options]
```

---

## Command Categories

| Category | Commands |
|----------|----------|
| Key management | `key generate`, `key rotate`, `key list` |
| Token operations | `token sign`, `token verify`, `token decode`, `token refresh` |
| User management | `user create`, `user verify`, `user reset-password`, `user disable`, `user list` |
| Session management | `session list`, `session revoke`, `session revoke-all` |
| OAuth management | `oauth list-providers`, `oauth add`, `oauth remove`, `oauth link` |
| Diagnostics | `audit list`, `stats auth`, `health check` |

---

## JSON Payload Format

All commands that accept input support a `--json` flag:

```bash
rok-auth token sign --json '{"sub": "user-123", "roles": ["admin"]}'
rok-auth user create --json '{"email": "alice@example.com", "password": "hunter2"}'
```

---

## Key Management

```bash
# Generate a new signing key
rok-auth key generate --output ./keys/signing.key

# Rotate the active key (keeps old key for verification)
rok-auth key rotate --config ./rok-auth.toml

# List all keys and their status
rok-auth key list
```

---

## Token Operations

```bash
# Sign a token
rok-auth token sign --sub user-123 --roles admin,user

# Verify a token (exits 0 if valid, 1 if invalid)
rok-auth token verify --token "eyJ..."

# Decode without verifying (inspect claims)
rok-auth token decode --token "eyJ..."

# Exchange a refresh token for a new pair
rok-auth token refresh --token "eyJ..."
```

---

## User Management

```bash
rok-auth user create --email alice@example.com --password hunter2 --roles user
rok-auth user verify --email alice@example.com
rok-auth user reset-password --email alice@example.com
rok-auth user disable --id user-123
rok-auth user list --role admin
```

---

## Session Management

```bash
rok-auth session list --user user-123
rok-auth session revoke --session-id sess-abc123
rok-auth session revoke-all --user user-123
```

---

## Diagnostics

```bash
rok-auth health check          # component health status
rok-auth stats auth            # login metrics
rok-auth audit list --days 7   # recent audit events
```
