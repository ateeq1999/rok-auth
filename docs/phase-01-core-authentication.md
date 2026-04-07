# Phase 1: Core Authentication Foundation

## Overview

Establish the core JWT authentication infrastructure that will underpin all subsequent features.

## Features

### 1.1 JWT Implementation ✅
- [x] Access token signing with configurable algorithm (HS256 default)
- [x] Refresh token generation and validation
- [x] Token expiration handling
- [x] Issuer (iss) and audience (aud) claims support

### 1.2 Claims Management ✅
- [x] Standard JWT claims (sub, exp, iat, iss)
- [x] Custom claims for user roles
- [x] Role-based claim helpers (has_role, has_any_role, has_all_roles)
- [x] Claims serialization/deserialization

### 1.3 Configuration ✅
- [x] AuthConfig with secret, TTLs, issuer
- [x] Environment-based configuration support
- [x] Validation of required fields

### 1.4 Error Types ✅
- [x] AuthError enum with all error variants
- [x] User-friendly error messages
- [x] Error code mapping

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
- [ ] In Progress
- [x] Completed
