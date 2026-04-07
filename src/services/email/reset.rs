//! Password reset token generation and validation.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetToken {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub token_type: ResetTokenType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ResetTokenType {
    PasswordReset,
    EmailChange,
    AccountRecovery,
}

impl ResetToken {
    pub fn new(
        user_id: String,
        email: String,
        ttl_minutes: i64,
        token_type: ResetTokenType,
    ) -> Self {
        let now = Utc::now();
        Self {
            token: generate_secure_token(),
            user_id,
            email,
            created_at: now,
            expires_at: now + Duration::minutes(ttl_minutes),
            used: false,
            token_type,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.used && Utc::now() < self.expires_at
    }

    pub fn mark_used(&mut self) {
        self.used = true;
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

fn generate_secure_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 48];
    rand::thread_rng().fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}_{}", hex[..32].to_string(), hex[32..].to_string())
}

pub struct ResetService {
    ttl_minutes: i64,
    tokens: std::sync::Arc<tokio::sync::RwLock<Vec<ResetToken>>>,
    #[allow(dead_code)]
    max_attempts: usize,
}

impl ResetService {
    pub fn new(ttl_minutes: i64, max_attempts: usize) -> Self {
        Self {
            ttl_minutes,
            tokens: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            max_attempts,
        }
    }

    pub async fn create_password_reset(&self, user_id: String, email: String) -> ResetToken {
        let token = ResetToken::new(
            user_id,
            email,
            self.ttl_minutes,
            ResetTokenType::PasswordReset,
        );
        let mut tokens = self.tokens.write().await;
        tokens.push(token.clone());
        token
    }

    pub async fn create_email_change(&self, user_id: String, new_email: String) -> ResetToken {
        let token = ResetToken::new(
            user_id,
            new_email,
            self.ttl_minutes,
            ResetTokenType::EmailChange,
        );
        let mut tokens = self.tokens.write().await;
        tokens.push(token.clone());
        token
    }

    pub async fn create_recovery_token(&self, user_id: String, email: String) -> ResetToken {
        let token = ResetToken::new(
            user_id,
            email,
            self.ttl_minutes,
            ResetTokenType::AccountRecovery,
        );
        let mut tokens = self.tokens.write().await;
        tokens.push(token.clone());
        token
    }

    pub async fn verify_and_consume(&self, token_str: &str) -> Result<ResetToken, ResetError> {
        let mut tokens = self.tokens.write().await;

        let token = tokens
            .iter_mut()
            .find(|t| t.token == token_str)
            .ok_or(ResetError::TokenNotFound)?;

        if token.used {
            return Err(ResetError::TokenAlreadyUsed);
        }
        if token.is_expired() {
            return Err(ResetError::TokenExpired);
        }

        token.mark_used();
        Ok(token.clone())
    }

    pub async fn invalidate_user_tokens(&self, user_id: &str) {
        let mut tokens = self.tokens.write().await;
        for token in tokens.iter_mut() {
            if token.user_id == user_id {
                token.mark_used();
            }
        }
    }

    pub async fn cleanup_expired(&self) {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        tokens.retain(|t| !t.used && t.expires_at > now);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResetError {
    #[error("reset token not found")]
    TokenNotFound,
    #[error("reset token has already been used")]
    TokenAlreadyUsed,
    #[error("reset token has expired")]
    TokenExpired,
    #[error("too many reset attempts")]
    TooManyAttempts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_verify_password_reset() {
        let service = ResetService::new(60, 3);
        let token = service
            .create_password_reset("user-123".to_string(), "test@example.com".to_string())
            .await;

        assert!(!token.token.is_empty());
        assert_eq!(token.token_type, ResetTokenType::PasswordReset);
        assert!(token.is_valid());

        let verified = service.verify_and_consume(&token.token).await.unwrap();
        assert_eq!(verified.user_id, "user-123");
    }

    #[tokio::test]
    async fn token_cannot_be_used_twice() {
        let service = ResetService::new(60, 3);
        let token = service
            .create_password_reset("user-123".to_string(), "test@example.com".to_string())
            .await;

        service.verify_and_consume(&token.token).await.unwrap();
        let result = service.verify_and_consume(&token.token).await;

        assert!(matches!(result, Err(ResetError::TokenAlreadyUsed)));
    }

    #[tokio::test]
    async fn expired_token_rejected() {
        let service = ResetService::new(0, 3);
        let token = service
            .create_password_reset("user-123".to_string(), "test@example.com".to_string())
            .await;

        let result = service.verify_and_consume(&token.token).await;
        assert!(matches!(result, Err(ResetError::TokenExpired)));
    }

    #[tokio::test]
    async fn invalidate_all_user_tokens() {
        let service = ResetService::new(60, 3);
        let token1 = service
            .create_password_reset("user-123".to_string(), "test@example.com".to_string())
            .await;
        let token2 = service
            .create_email_change("user-123".to_string(), "new@example.com".to_string())
            .await;

        service.invalidate_user_tokens("user-123").await;

        let result1 = service.verify_and_consume(&token1.token).await;
        let result2 = service.verify_and_consume(&token2.token).await;

        assert!(matches!(result1, Err(ResetError::TokenAlreadyUsed)));
        assert!(matches!(result2, Err(ResetError::TokenAlreadyUsed)));
    }
}
