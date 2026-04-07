//! Email verification token generation and verification.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

impl VerificationToken {
    pub fn new(user_id: String, email: String, ttl_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            token: generate_token(),
            user_id,
            email,
            created_at: now,
            expires_at: now + Duration::hours(ttl_hours),
            used: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.used && Utc::now() < self.expires_at
    }

    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub struct VerificationService {
    ttl_hours: i64,
    tokens: std::sync::Arc<tokio::sync::RwLock<Vec<VerificationToken>>>,
}

impl VerificationService {
    pub fn new(ttl_hours: i64) -> Self {
        Self {
            ttl_hours,
            tokens: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn create_token(&self, user_id: String, email: String) -> VerificationToken {
        let token = VerificationToken::new(user_id, email, self.ttl_hours);
        let mut tokens = self.tokens.write().await;
        tokens.push(token.clone());
        token
    }

    pub async fn verify_token(&self, token_str: &str) -> Result<VerificationToken, VerificationError> {
        let mut tokens = self.tokens.write().await;
        let token = tokens
            .iter_mut()
            .find(|t| t.token == token_str)
            .ok_or(VerificationError::TokenNotFound)?;

        if token.used {
            return Err(VerificationError::TokenAlreadyUsed);
        }
        if Utc::now() >= token.expires_at {
            return Err(VerificationError::TokenExpired);
        }

        token.mark_used();
        Ok(token.clone())
    }

    pub async fn cleanup_expired(&self) {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        tokens.retain(|t| !t.used && t.expires_at > now);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("verification token not found")]
    TokenNotFound,
    #[error("verification token has already been used")]
    TokenAlreadyUsed,
    #[error("verification token has expired")]
    TokenExpired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_verify_token() {
        let service = VerificationService::new(24);
        let token = service.create_token("user-123".to_string(), "test@example.com".to_string()).await;
        
        assert!(!token.token.is_empty());
        assert_eq!(token.user_id, "user-123");
        assert!(token.is_valid());

        let verified = service.verify_token(&token.token).await.unwrap();
        assert_eq!(verified.user_id, "user-123");
    }

    #[tokio::test]
    async fn token_cannot_be_used_twice() {
        let service = VerificationService::new(24);
        let token = service.create_token("user-123".to_string(), "test@example.com".to_string()).await;
        
        service.verify_token(&token.token).await.unwrap();
        let result = service.verify_token(&token.token).await;
        
        assert!(matches!(result, Err(VerificationError::TokenAlreadyUsed)));
    }

    #[tokio::test]
    async fn expired_token_rejected() {
        let service = VerificationService::new(0);
        let token = service.create_token("user-123".to_string(), "test@example.com".to_string()).await;
        
        let result = service.verify_token(&token.token).await;
        assert!(matches!(result, Err(VerificationError::TokenExpired)));
    }

    #[tokio::test]
    async fn invalid_token_rejected() {
        let service = VerificationService::new(24);
        let result = service.verify_token("invalid-token").await;
        assert!(matches!(result, Err(VerificationError::TokenNotFound)));
    }
}
