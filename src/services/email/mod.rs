//! Email services for verification and password reset.

pub mod reset;
pub mod sender;
pub mod templates;
pub mod verification;

pub use reset::{ResetError, ResetService, ResetToken, ResetTokenType};
pub use sender::{
    ConsoleEmailSender, Email, EmailSender, NoopEmailSender, SendEmailError, SmtpConfig,
};
pub use templates::{EmailTemplate, TemplateEngine};
pub use verification::{VerificationError, VerificationService, VerificationToken};
