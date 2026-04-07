# Phase 1: Core Authentication Foundation

## Overview

Establish the core JWT authentication infrastructure that will underpin all subsequent features.

## Features

### 1.1 JWT Implementation
- [ ] Access token signing with configurable algorithm (HS256 default)
- [ ] Refresh token generation and validation
- [ ] Token expiration handling
- [ ] Issuer (iss) and audience (aud) claims support

### 1.2 Claims Management
- [ ] Standard JWT claims (sub, exp, iat, iss)
- [ ] Custom claims for user roles
- [ ] Role-based claim helpers (has_role, has_any_role, has_all_roles)
- [ ] Claims serialization/deserialization

### 1.3 Configuration
- [ ] AuthConfig with secret, TTLs, issuer
- [ ] Environment-based configuration support
- [ ] Validation of required fields

### 1.4 Error Types
- [ ] AuthError enum with all error variants
- [ ] User-friendly error messages
- [ ] Error code mapping

## File Structure

```
src/
├── claims.rs        (< 100 lines)
├── config.rs        (< 50 lines)
├── error.rs         (< 50 lines)
├── jwt.rs           (< 150 lines)
└── lib.rs           (< 50 lines)
```

## Acceptance Criteria

1. JWTs can be signed and verified with configurable secret
2. Claims include sub, exp, iat, iss, roles
3. Role checking methods work correctly
4. Errors are properly typed and handled
5. All tests pass

## Dependencies

- jsonwebtoken
- serde
- chrono

## Status

- [ ] Not Started
- [x] In Progress
- [ ] Completed
