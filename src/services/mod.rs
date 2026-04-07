//! Built-in authentication services.

mod session;
mod oauth;
pub mod totp;
pub mod email;

pub use session::SessionService;
pub use totp::{TotpService, TotpConfig, TotpCode, TotpError, BackupCodes, BackupCode};

pub use oauth::{
    OAuthConfig, OAuthError, OAuthProvider, OAuthService, OAuthTokens, OAuthUserInfo,
    AuthorizationUrl, GoogleProvider, GitHubProvider, DiscordProvider,
};

pub use email::{
    VerificationToken, VerificationService, VerificationError,
    ResetToken, ResetService, ResetError, ResetTokenType,
    EmailTemplate, TemplateEngine,
    Email, SmtpConfig, EmailSender, SendEmailError,
    ConsoleEmailSender, NoopEmailSender,
};
