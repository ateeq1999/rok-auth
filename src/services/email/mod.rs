//! Email services for verification and password reset.

pub mod verification;
pub mod reset;
pub mod templates;
pub mod sender;

pub use verification::{VerificationToken, VerificationService, VerificationError};
pub use reset::{ResetToken, ResetService, ResetError, ResetTokenType};
pub use templates::{EmailTemplate, TemplateEngine};
pub use sender::{Email, SmtpConfig, EmailSender, SendEmailError, ConsoleEmailSender, NoopEmailSender};
