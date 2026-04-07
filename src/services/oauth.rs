//! OAuth 2.0 provider support for social authentication.
//!
//! This module provides a generic OAuth provider interface and implementations
//! for popular OAuth providers like Google, GitHub, and Discord.
//!
//! # Example
//!
//! ```rust,no_run
//! use rok_auth::services::{OAuthService, GoogleProvider};
//!
//! let google = GoogleProvider::new(
//!     "client_id".to_string(),
//!     "client_secret".to_string(),
//!     "https://example.com/callback".to_string(),
//! );
//!
//! let service = OAuthService::new(google);
//! let auth = service.authorization_url().unwrap();
//! ```

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        auth_url: String,
        token_url: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url,
            token_url,
            scopes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub id_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct AuthorizationUrl {
    pub url: String,
    pub state: String,
}

pub trait OAuthProvider: Send + Sync {
    fn config(&self) -> &OAuthConfig;
    fn provider_name(&self) -> &'static str;
    fn default_scopes(&self) -> Vec<String>;

    fn authorization_url(&self, state: &str) -> Result<String, OAuthError> {
        let config = self.config();
        let scopes = if config.scopes.is_empty() {
            self.default_scopes()
        } else {
            config.scopes.clone()
        };

        let mut url = Url::parse(&config.auth_url).map_err(|_| OAuthError::InvalidUrl)?;
        url.query_pairs_mut()
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &scopes.join(" "))
            .append_pair("state", state);

        Ok(url.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("invalid URL")]
    InvalidUrl,
    #[error("token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("user info retrieval failed: {0}")]
    UserInfoFailed(String),
    #[error("invalid state parameter")]
    InvalidState,
    #[error("missing required field: {0}")]
    MissingField(String),
}

pub struct GoogleProvider {
    config: OAuthConfig,
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            config: OAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
        }
    }
}

impl OAuthProvider for GoogleProvider {
    fn config(&self) -> &OAuthConfig {
        &self.config
    }

    fn provider_name(&self) -> &'static str {
        "google"
    }

    fn default_scopes(&self) -> Vec<String> {
        vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ]
    }
}

pub struct GitHubProvider {
    config: OAuthConfig,
}

impl GitHubProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            config: OAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                scopes: vec!["read:user".to_string(), "user:email".to_string()],
            },
        }
    }
}

impl OAuthProvider for GitHubProvider {
    fn config(&self) -> &OAuthConfig {
        &self.config
    }

    fn provider_name(&self) -> &'static str {
        "github"
    }

    fn default_scopes(&self) -> Vec<String> {
        vec!["read:user".to_string(), "user:email".to_string()]
    }
}

pub struct DiscordProvider {
    config: OAuthConfig,
}

impl DiscordProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            config: OAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                auth_url: "https://discord.com/oauth2/authorize".to_string(),
                token_url: "https://discord.com/api/oauth2/token".to_string(),
                scopes: vec!["identify".to_string(), "email".to_string()],
            },
        }
    }
}

impl OAuthProvider for DiscordProvider {
    fn config(&self) -> &OAuthConfig {
        &self.config
    }

    fn provider_name(&self) -> &'static str {
        "discord"
    }

    fn default_scopes(&self) -> Vec<String> {
        vec!["identify".to_string(), "email".to_string()]
    }
}

pub struct OAuthService<P: OAuthProvider> {
    provider: P,
}

impl<P: OAuthProvider> OAuthService<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn authorization_url(&self) -> Result<AuthorizationUrl, OAuthError> {
        let state = generate_state();
        let url = self.provider.authorization_url(&state)?;
        Ok(AuthorizationUrl { url, state })
    }

    pub async fn exchange_code(&self, code: &str, _state: &str) -> Result<OAuthTokens, OAuthError> {
        let config = self.provider.config();

        let params = [
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];

        let client = reqwest::Client::new();
        let response = client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| OAuthError::RequestFailed(e.to_string()))?;

        let tokens: OAuthTokens = response
            .json()
            .await
            .map_err(|e| OAuthError::TokenExchangeFailed(e.to_string()))?;

        Ok(tokens)
    }

    pub async fn get_user_info(&self, tokens: &OAuthTokens) -> Result<OAuthUserInfo, OAuthError> {
        let user_info_url = match self.provider.provider_name() {
            "google" => "https://www.googleapis.com/oauth2/v2/userinfo",
            "github" => "https://api.github.com/user",
            "discord" => "https://discord.com/api/users/@me",
            _ => return Err(OAuthError::InvalidUrl),
        };

        let client = reqwest::Client::new();
        let mut request = client.get(user_info_url);
        request = request.header("Authorization", format!("Bearer {}", tokens.access_token));

        if self.provider.provider_name() == "github" {
            request = request.header("Accept", "application/json");
        }

        let response = request
            .send()
            .await
            .map_err(|e| OAuthError::RequestFailed(e.to_string()))?;

        let user_info = parse_user_info(response, self.provider.provider_name()).await?;
        Ok(user_info)
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }
}

fn generate_state() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

async fn parse_user_info(
    response: reqwest::Response,
    provider: &str,
) -> Result<OAuthUserInfo, OAuthError> {
    match provider {
        "google" => {
            #[derive(Deserialize)]
            struct GoogleUser {
                id: String,
                email: Option<String>,
                name: Option<String>,
                picture: Option<String>,
            }
            let user: GoogleUser = response
                .json()
                .await
                .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;
            Ok(OAuthUserInfo {
                id: user.id,
                email: user.email,
                name: user.name,
                picture: user.picture,
                provider: provider.to_string(),
            })
        }
        "github" => {
            #[derive(Deserialize)]
            struct GitHubUser {
                id: u64,
                email: Option<String>,
                name: Option<String>,
                avatar_url: Option<String>,
            }
            let user: GitHubUser = response
                .json()
                .await
                .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;
            Ok(OAuthUserInfo {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                picture: user.avatar_url,
                provider: provider.to_string(),
            })
        }
        "discord" => {
            #[derive(Deserialize)]
            struct DiscordUser {
                id: String,
                email: Option<String>,
                global_name: Option<String>,
                avatar: Option<String>,
            }
            let user: DiscordUser = response
                .json()
                .await
                .map_err(|e| OAuthError::UserInfoFailed(e.to_string()))?;
            let picture = user.avatar.map(|avatar_id| {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png",
                    user.id, avatar_id
                )
            });
            Ok(OAuthUserInfo {
                id: user.id,
                email: user.email,
                name: user.global_name,
                picture,
                provider: provider.to_string(),
            })
        }
        _ => Err(OAuthError::UserInfoFailed("Unknown provider".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_provider_auth_url() {
        let google = GoogleProvider::new(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = google.authorization_url("test_state").unwrap();
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("scope="));
    }

    #[test]
    fn github_provider_auth_url() {
        let github = GitHubProvider::new(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = github.authorization_url("test_state").unwrap();
        assert!(url.contains("github.com"));
        assert!(url.contains("client_id=test_client_id"));
    }

    #[test]
    fn discord_provider_auth_url() {
        let discord = DiscordProvider::new(
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost/callback".to_string(),
        );

        let url = discord.authorization_url("test_state").unwrap();
        assert!(url.contains("discord.com"));
    }

    #[test]
    fn state_is_generated() {
        let state1 = generate_state();
        let state2 = generate_state();
        assert_ne!(state1, state2);
        assert_eq!(state1.len(), 64);
    }

    #[test]
    fn oauth_service_authorization_url() {
        let google = GoogleProvider::new(
            "client_id".to_string(),
            "secret".to_string(),
            "http://localhost".to_string(),
        );
        let service = OAuthService::new(google);
        let auth = service.authorization_url().unwrap();
        assert!(!auth.url.is_empty());
        assert_eq!(auth.state.len(), 64);
    }
}
