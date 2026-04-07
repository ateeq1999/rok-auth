//! Built-in authentication services.

mod session;
mod oauth;
pub mod totp;

pub use session::SessionService;
pub use totp::{TotpService, TotpConfig, TotpCode, TotpError, BackupCodes, BackupCode};

pub use oauth::{
    OAuthConfig, OAuthError, OAuthProvider, OAuthService, OAuthTokens, OAuthUserInfo,
    AuthorizationUrl, GoogleProvider, GitHubProvider, DiscordProvider,
};
