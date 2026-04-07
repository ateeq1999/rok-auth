# Phase 3: Two-Factor Authentication (TOTP)

## Overview

Add time-based one-time password (TOTP) support for enhanced account security.

## Features

### 3.1 TOTP Generation
- [ ] TOTP secret generation
- [ ] QR code generation for authenticator apps
- [ ] Base32 encoding support
- [ ] Configurable time step and digits

### 3.2 TOTP Verification
- [ ] Code validation with tolerance window
- [ ] Timing attack prevention
- [ ] Backup codes support

### 3.3 2FA Management
- [ ] Enable/disable 2FA for users
- [ ] 2FA setup flow
- [ ] Recovery codes generation

### 3.4 Integration
- [ ] 2FA requirement checking in auth flows
- [ ] Remember device functionality
- [ ] 2FA bypass codes for recovery

## File Structure

```
src/services/
├── mod.rs
├── totp.rs           (< 200 lines)
└── backup_codes.rs   (< 100 lines)
```

## Acceptance Criteria

1. TOTP secrets can be generated and verified
2. QR codes are valid for authenticator apps
3. Backup codes work for account recovery
4. All tests pass

## Dependencies

- totp (crate)
- base32 (crate)
- qrcode (crate, optional)

## Status

- [ ] Not Started
- [ ] In Progress
- [ ] Completed
