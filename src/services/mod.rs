//! Built-in authentication services.

pub mod email;
mod oauth;
mod session;
pub mod totp;

pub use session::SessionService;
pub use totp::{BackupCode, BackupCodes, TotpCode, TotpConfig, TotpError, TotpService};

pub use oauth::{
    AuthorizationUrl, DiscordProvider, GitHubProvider, GoogleProvider, OAuthConfig, OAuthError,
    OAuthProvider, OAuthService, OAuthTokens, OAuthUserInfo,
};

pub use email::{
    ConsoleEmailSender, Email, EmailSender, EmailTemplate, NoopEmailSender, ResetError,
    ResetService, ResetToken, ResetTokenType, SendEmailError, SmtpConfig, TemplateEngine,
    VerificationError, VerificationService, VerificationToken,
};
