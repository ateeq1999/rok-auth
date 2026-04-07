# Phase 4: OAuth Integration

## Overview

Implement OAuth 2.0 provider support for social authentication.

## Features

### 4.1 OAuth Provider Trait
- [ ] Generic OAuthProvider trait
- [ ] Configuration for OAuth providers
- [ ] Provider-specific implementations

### 4.2 OAuth Flows
- [ ] Authorization URL generation
- [ ] Token exchange
- [ ] User info retrieval
- [ ] State parameter validation

### 4.3 Supported Providers
- [ ] Google OAuth
- [ ] GitHub OAuth
- [ ] Discord OAuth
- [ ] Extensible for more providers

### 4.4 Account Linking
- [ ] Link OAuth accounts to existing users
- [ ] Auto-registration with OAuth
- [ ] Multiple providers per user

## File Structure

```
src/services/
├── mod.rs
├── oauth/
│   ├── mod.rs         (< 30 lines)
│   ├── provider.rs    (< 100 lines)
│   ├── google.rs       (< 100 lines)
│   ├── github.rs       (< 100 lines)
│   └── discord.rs      (< 100 lines)
```

## Acceptance Criteria

1. OAuth authorization flows work correctly
2. Tokens are exchanged properly
3. User info is retrieved from providers
4. Account linking functions properly
5. All tests pass

## Dependencies

- reqwest (for HTTP)
- url (for URL manipulation)
- serde

## Status

- [ ] Not Started
- [ ] In Progress
- [ ] Completed
