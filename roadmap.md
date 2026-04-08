# rok-auth Project Roadmap

A comprehensive authentication library for Rust, inspired by [better-auth](https://better-auth.com/), providing built-in authentication services that integrate seamlessly with your application.

## Overview

rok-auth is a production-ready authentication library that provides:
- JWT authentication with access/refresh tokens
- Password hashing with Argon2id
- Session management
- OAuth provider integration
- Two-factor authentication (TOTP)
- Email verification and password reset
- Role-based access control (RBAC)
- Rate limiting and security hardening

## Implementation Guidelines

### File Organization

All code lives under `rok-auth/src/` with the following structure:

```
src/
├── main.rs              # Binary entry (CLI)
├── lib.rs               # Library entry point
├── claims.rs            # JWT claims
├── config.rs            # Configuration
├── error.rs             # Error types
├── jwt.rs               # JWT operations
├── builders.rs          # Builder patterns (Phase 7)
├── utils.rs             # Utility functions (Phase 7)
├── password/
│   ├── mod.rs
│   └── hash.rs
├── providers/
│   ├── mod.rs
│   └── trait_.rs
├── authorization/       # RBAC (Phase 6)
│   ├── mod.rs
│   ├── roles.rs
│   ├── permissions.rs
│   ├── policies.rs
│   └── audit.rs
├── services/
│   ├── mod.rs
│   ├── oauth.rs         # (Phase 4)
│   ├── session.rs
│   ├── totp.rs         # (Phase 3)
│   └── email/          # (Phase 5)
│       ├── mod.rs
│       ├── verification.rs
│       ├── reset.rs
│       ├── templates.rs
│       └── sender.rs
├── security/           # Rate limiting (Phase 8)
│   ├── mod.rs
│   ├── rate_limiter.rs
│   ├── headers.rs
│   ├── detection.rs
│   ├── cors.rs
│   └── health.rs
├── session/
│   ├── mod.rs
│   └── token.rs
├── tokens/
│   ├── mod.rs
│   ├── pair.rs
│   └── refresh.rs
└── web/
    ├── mod.rs
    └── axum/
        ├── mod.rs
        ├── layer.rs
        ├── extractor.rs
        ├── guard.rs
        └── error.rs

rok-auth-macros/        # Procedural macros (Phase 7)
└── src/
    └── lib.rs

docs/
├── phase-01-core-authentication.md
├── phase-02-user-authentication.md
├── phase-03-two-factor-auth.md
├── phase-04-oauth-integration.md
├── phase-05-email-verification.md
├── phase-06-rbac-advanced.md
├── phase-07-api-polish.md
├── phase-08-rate-limiting-security.md
├── phase-09-cli-commands.md
├── phase-10-security-enhancements.md
├── dev.md
├── commands.md
└── progress.md
```

### Code Style Guidelines

1. **File Size Limit**: No file should exceed 400 lines of code
2. **Module Organization**: Split large modules into submodules
3. **Documentation**: All public APIs must have doc comments
4. **Tests**: Each module should have inline tests
5. **Error Handling**: Use `thiserror` for error types

### Module Documentation Template

```rust
//! Module name and brief description.
//!
//! Detailed explanation of the module's purpose and usage.
//!
//! # Example
//!
//! ```rust,no_run
//! use rok_auth::{Type, function};
//!
//! // Basic usage example
//! ```

### Phase Workflow

1. Read the phase documentation in `docs/`
2. Implement features according to the specification
3. Ensure all files are under 400 lines
4. Write/update tests
5. Run linting and type checking
6. Commit with phase tag: `git commit -m "Phase N: <description>"`
7. Update `docs/progress.md`

### Testing Requirements

- Unit tests in each module (`#[cfg(test)]` block)
- Integration tests for complex features
- All tests must pass before committing

### Linting & Type Checking

Before each commit, run:

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo check --all-features
```

---

## Phases

| Phase | Title | Status | Dependencies |
|-------|-------|--------|--------------|
| 1 | Core Authentication Foundation | Completed | None |
| 2 | User Authentication Flows | Completed | Phase 1 |
| 3 | Two-Factor Authentication (TOTP) | Completed | Phase 2 |
| 4 | OAuth Integration | Completed | Phase 2 |
| 5 | Email Verification & Account Recovery | Completed | Phase 2 |
| 6 | Advanced RBAC & Authorization | Completed | Phase 1, 2 |
| 7 | API Polish & Developer Experience | Completed | Phase 1-6 |
| 8 | Rate Limiting & Security Hardening | Completed | Phase 1-7 |
| 9 | CLI Commands Specification | Completed | Phase 1-8 |
| 10 | Security Enhancements | In Progress | Phase 1-9 |

### Phase Dependencies Graph

```
Phase 1 ──┬── Phase 2 ──┬── Phase 3
          │             ├── Phase 4
          │             ├── Phase 5
          │             │
          └── Phase 6 ──┘
                        │
Phase 7 ────────────────┘
                        │
Phase 8 ────────────────┘
                        │
Phase 9 ────────────────┘
                        │
Phase 10 ───────────────┘
```

---

## Phase 10: Security Enhancements

Inspired by Laravel Sanctum and JWT best practices, this phase adds advanced security features to the authentication library.

### Features

| Feature | Priority | Complexity | Implementation Location |
|---------|----------|------------|------------------------|
| Token Abilities/Scopes | High | Low | `src/tokens/abilities.rs` |
| Token Blacklist | High | Medium | `src/security/blacklist.rs` |
| Device Tracking | Medium | Medium | `src/security/device.rs` |
| Strict Algorithm Validation | High | Low | `src/jwt/strict.rs` |
| CSRF Protection | Medium | Medium | `src/security/csrf.rs` |
| Step-up Authentication | Medium | Low | `src/security/step_up.rs` |
| Structured Audit Events | Low | Low | `src/authorization/audit.rs` |
| Security Webhooks | Low | Medium | `src/security/webhook.rs` |

### Implementation Notes

1. All features should be optional and configurable via feature flags
2. Default configurations should be secure by default
3. Database-related features require the database layer integration
4. Each module should have comprehensive inline tests

### File Structure Additions

```
src/
├── tokens/
│   └── abilities.rs      # Token abilities/scopes (new)
├── security/
│   ├── blacklist.rs     # Token revocation (new)
│   ├── device.rs        # Device tracking (new)
│   ├── csrf.rs          # CSRF protection (new)
│   ├── step_up.rs       # Step-up authentication (new)
│   └── webhook.rs      # Security webhooks (new)
├── jwt/
│   └── strict.rs        # Strict algorithm validation (new)
└── authorization/
    └── audit.rs         # Enhanced audit events (extend)
```

---

## CLI Integration

CLI commands are implemented in the `rok-cli` crate. See `docs/commands.md` for detailed specifications.

### Command Categories

1. **Key Management**: `key generate`, `key rotate`, `key list`
2. **Token Operations**: `token sign`, `token verify`, `token decode`, `token refresh`
3. **User Management**: `user create`, `user verify`, `user reset-password`, `user disable`, `user list`
4. **Session Management**: `session list`, `session revoke`, `session revoke-all`
5. **OAuth Management**: `oauth list-providers`, `oauth add`, `oauth remove`, `oauth link`
6. **Audit & Monitoring**: `audit list`, `stats auth`, `health check`

### JSON Payload Format

All commands support JSON payload:

```bash
rok-cli <command> --json '{
  "key": "value"
}'
```

---

## Progress Tracking

Current implementation progress is tracked in `docs/progress.md`.

To update progress:

1. Mark completed items in phase documentation
2. Update `docs/progress.md` with completion percentage
3. Record any blockers or notes

---

## Getting Started

### Prerequisites

- Rust 1.75+
- Cargo workspace support

### Build

```bash
cargo build --all-features
```

### Test

```bash
cargo test --all-features
```

### Lint

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
```

---

## Contributing

1. Pick an unstarted phase or task
2. Read the phase documentation thoroughly
3. Implement according to guidelines
4. Ensure code is under 400 lines per file
5. Write tests
6. Run linting
7. Commit with phase prefix
8. Update progress

---

## License

MIT OR Apache-2.0
