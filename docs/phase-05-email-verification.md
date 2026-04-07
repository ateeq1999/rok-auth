# Phase 5: Email Verification & Account Recovery

## Overview

Implement email verification and account recovery flows.

## Features

### 5.1 Email Verification
- [ ] Verification token generation
- [ ] Email sending abstraction
- [ ] Verification status tracking
- [ ] Resend functionality

### 5.2 Password Reset
- [ ] Reset token generation
- [ ] Secure token validation
- [ ] Password update flow
- [ ] Token expiration

### 5.3 Email Templates
- [ ] Verification email template
- [ ] Password reset email template
- [ ] Customizable templates
- [ ] HTML/text multipart support

### 5.4 Rate Limiting
- [ ] Verification email rate limits
- [ ] Password reset rate limits
- [ ] Account lockout protection

## File Structure

```
src/services/
├── mod.rs
├── email/
│   ├── mod.rs         (< 30 lines)
│   ├── verification.rs (< 150 lines)
│   ├── reset.rs        (< 150 lines)
│   └── templates.rs    (< 100 lines)
```

## Acceptance Criteria

1. Verification tokens are secure and unique
2. Email sending is abstracted for flexibility
3. Password reset flows are secure
4. Rate limiting prevents abuse
5. All tests pass

## Dependencies

- uuid
- rand
- tokio

## Status

- [ ] Not Started
- [ ] In Progress
- [ ] Completed
