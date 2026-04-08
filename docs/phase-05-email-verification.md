# Phase 5: Email Verification & Account Recovery

Email-based account verification and password reset, implemented in [src/services/email/](../src/services/email/).

---

## Modules

| File | Exports |
|------|---------|
| `verification.rs` | `VerificationService`, `VerificationToken`, `VerificationError` |
| `reset.rs` | `ResetService`, `ResetToken`, `ResetTokenType`, `ResetError` |
| `sender.rs` | `EmailSender` trait, `Email`, `ConsoleEmailSender`, `NoopEmailSender`, `SmtpConfig` |
| `templates.rs` | `EmailTemplate`, `TemplateEngine` |

---

## Email Senders

The `EmailSender` trait abstracts the actual delivery mechanism. rok-auth ships two built-in implementations:

### ConsoleEmailSender

Prints emails to stdout — useful during development.

```rust
use rok_auth::services::email::ConsoleEmailSender;

let sender = ConsoleEmailSender;
```

### NoopEmailSender

Silently discards all emails — useful in tests.

```rust
use rok_auth::services::email::NoopEmailSender;

let sender = NoopEmailSender;
```

### Custom SMTP

Implement `EmailSender` for your SMTP client or transactional email provider (SendGrid, Mailgun, etc.):

```rust
use rok_auth::services::email::{Email, EmailSender, SendEmailError};

struct MySendgrid { api_key: String }

impl EmailSender for MySendgrid {
    async fn send(&self, email: Email) -> Result<(), SendEmailError> {
        // call SendGrid API
        Ok(())
    }
}
```

---

## Email Verification

```rust
use rok_auth::services::email::{VerificationService, ConsoleEmailSender};

let service = VerificationService::new(ConsoleEmailSender);

// 1. On registration — generate and send a verification email
let token = service.send_verification("alice@example.com").await?;
// Store token.value in your DB linked to the user

// 2. On link click — verify the token the user submitted
service.verify_token(token.value()).await?;
// If Ok, mark the user's email as verified in your DB
```

`VerificationToken` contains a random, time-limited token. Expired tokens return `VerificationError::Expired`.

---

## Password Reset

```rust
use rok_auth::services::email::{ResetService, ConsoleEmailSender};

let service = ResetService::new(ConsoleEmailSender);

// 1. User submits their email
let token = service.send_reset("alice@example.com").await?;
// Store token.value in your DB; do NOT confirm whether the email exists

// 2. User follows the link and submits a new password
service.verify_reset_token(token.value()).await?;
// If Ok, hash the new password and update in your DB
// Then invalidate the token
```

### Reset token types

```rust
pub enum ResetTokenType {
    PasswordReset,
    EmailChange,
    AccountDeletion,
}
```

---

## Email Templates

`TemplateEngine` renders HTML and plain-text bodies using `EmailTemplate`:

```rust
use rok_auth::services::email::{EmailTemplate, TemplateEngine};

let engine = TemplateEngine::new();

let html = engine.render(EmailTemplate::Verification {
    verification_url: "https://example.com/verify?token=abc".to_string(),
    user_name: "Alice".to_string(),
})?;
```

Built-in templates: `Verification`, `PasswordReset`, `WelcomeEmail`.

---

## Security Notes

- Always respond with a generic success message for password reset, even if the email does not exist (prevents user enumeration)
- Set short expiry on all tokens: 24 hours for verification, 1 hour for password reset
- Invalidate a reset token immediately after use — single-use only
- Send emails from a dedicated transactional domain with SPF, DKIM, and DMARC configured
