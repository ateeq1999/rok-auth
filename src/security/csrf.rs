//! CSRF token generation and validation.
//!
//! Provides CSRF protection inspired by Laravel Sanctum's cookie flow.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CsrfProtection {
    store: Arc<tokio::sync::Mutex<HashMap<String, DateTime<Utc>>>>,
    token_ttl_secs: u64,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            store: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            token_ttl_secs: 7200,
        }
    }

    pub fn with_ttl(mut self, secs: u64) -> Self {
        self.token_ttl_secs = secs;
        self
    }

    pub async fn generate(&self, session_id: &str) -> String {
        use base32::Alphabet;

        let alphabet = Alphabet::Rfc4648 { padding: false };
        let data1 = rand::random::<[u8; 16]>();
        let data2 = rand::random::<[u8; 16]>();
        let part1 = base32::encode(alphabet, &data1);
        let part2 = base32::encode(alphabet, &data2);

        let token = format!("{}.{}", part1, part2);

        let expiry = Utc::now() + chrono::Duration::seconds(self.token_ttl_secs as i64);
        let key = format!("{}:{}", session_id, &token);

        let mut store = self.store.lock().await;
        store.insert(key, expiry);

        token
    }

    pub async fn validate(&self, session_id: &str, token: &str) -> bool {
        let key = format!("{}:{}", session_id, token);
        let store = self.store.lock().await;

        if let Some(expires_at) = store.get(&key) {
            Utc::now() < *expires_at
        } else {
            false
        }
    }

    pub async fn cleanup(&self) {
        let mut store = self.store.lock().await;
        let now = Utc::now();
        store.retain(|_, exp| now < *exp);
    }

    pub async fn len(&self) -> usize {
        self.store.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.store.lock().await.is_empty()
    }
}

impl Default for CsrfProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CsrfProtection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CsrfProtection")
            .field("token_ttl_secs", &self.token_ttl_secs)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_token() {
        let csrf = CsrfProtection::new();
        let token = csrf.generate("session-123").await;

        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }

    #[tokio::test]
    async fn test_validate_valid_token() {
        let csrf = CsrfProtection::new();
        let session = "session-123";
        let token = csrf.generate(session).await;

        let is_valid = csrf.validate(session, &token).await;
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_validate_invalid_token() {
        let csrf = CsrfProtection::new();
        let session = "session-123";

        let is_valid = csrf.validate(session, "invalid.token").await;
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_validate_wrong_session() {
        let csrf = CsrfProtection::new();
        let token = csrf.generate("session-123").await;

        let is_valid = csrf.validate("session-456", &token).await;
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let csrf = CsrfProtection::new().with_ttl(1);
        let session = "session-123";

        csrf.generate(session).await;
        assert!(csrf.len().await > 0);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        csrf.cleanup().await;

        assert!(csrf.len().await == 0);
    }
}
