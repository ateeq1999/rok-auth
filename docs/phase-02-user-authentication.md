# Phase 2: User Authentication Flows

## Overview

Implement user authentication flows including registration, login, and session management.

## Features

### 2.1 Password Handling
- [ ] Argon2id password hashing
- [ ] Password verification
- [ ] Async variants for non-blocking operations
- [ ] Secure random salt generation

### 2.2 User Provider Trait
- [ ] UserProvider trait definition
- [ ] find_by_email implementation
- [ ] User ID, password hash, roles accessors
- [ ] Integration with user models

### 2.3 Token Management
- [ ] TokenPair for access/refresh token pairs
- [ ] Refresh token rotation
- [ ] Token exchange functionality
- [ ] Session token generation

### 2.4 Session Management
- [ ] Cryptographically secure session tokens
- [ ] Session storage interface
- [ ] Session validation

## File Structure

```
src/
├── password/
│   ├── mod.rs      (< 20 lines)
│   └── hash.rs     (< 80 lines)
├── session/
│   ├── mod.rs      (< 20 lines)
│   └── token.rs    (< 60 lines)
├── tokens/
│   ├── mod.rs      (< 20 lines)
│   ├── pair.rs     (< 20 lines)
│   └── refresh.rs  (< 40 lines)
├── providers/
│   ├── mod.rs      (< 20 lines)
│   └── trait_.rs   (< 40 lines)
```

## Acceptance Criteria

1. Passwords are hashed with Argon2id
2. UserProvider trait integrates with any user model
3. Token pairs are generated correctly
4. Session tokens are cryptographically secure
5. All tests pass

## Dependencies

- argon2
- rand

## Status

- [ ] Not Started
- [ ] In Progress
- [ ] Completed
