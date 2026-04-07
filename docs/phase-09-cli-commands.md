# Phase 9: CLI Commands Specification

## Overview

Define CLI commands for the rok-cli crate that interact with rok-auth.

## Commands

### 9.1 Key Management

```bash
rok-cli key generate [--algorithm <algo>] [--output <path>]
rok-cli key rotate --current <path> --new <path>
```

### 9.2 Token Operations

```bash
rok-cli token sign --sub <subject> [--roles <roles>] [--ttl <duration>]
rok-cli token verify --token <token>
rok-cli token decode --token <token>
rok-cli token refresh --refresh-token <token>
```

### 9.3 User Management

```bash
rok-cli user create --email <email> --password <password> [--roles <roles>]
rok-cli user verify --email <email>
rok-cli user reset-password --email <email>
rok-cli user disable --user-id <id>
rok-cli user list [--page <n>] [--limit <n>]
```

### 9.4 Session Management

```bash
rok-cli session list --user-id <id>
rok-cli session revoke --session-id <id>
rok-cli session revoke-all --user-id <id>
```

### 9.5 OAuth Management

```bash
rok-cli oauth list-providers
rok-cli oauth add --provider <name> --config <json>
rok-cli oauth remove --provider <name>
rok-cli oauth link --user-id <id> --provider <name>
```

### 9.6 Audit & Monitoring

```bash
rok-cli audit list [--user-id <id>] [--from <date>] [--to <date>]
rok-cli stats auth --period <duration>
rok-cli health check
```

## JSON Payload Format

All commands support JSON payload input:

```bash
rok-cli <command> --json-payload '{
  "sub": "user-123",
  "roles": ["admin", "user"],
  "ttl": "1h"
}'
```

## Flags

Common flags across all commands:
- `--json-payload <json>`: JSON input (agent-friendly)
- `--output-format <format>`: Output format (json, table, yaml)
- `--quiet`: Suppress output
- `--verbose`: Verbose logging
- `--config <path>`: Config file path

## Status

- [ ] Not Started
- [ ] In Progress
- [x] Completed
