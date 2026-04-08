//! Token blacklist for immediate token revocation.
//!
//! Provides JWT token revocation strategy by blacklisting token IDs (jti).

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TokenBlacklist {
    store: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    cleanup_interval_secs: u64,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval_secs: 3600,
        }
    }

    pub fn with_cleanup_interval(mut self, secs: u64) -> Self {
        self.cleanup_interval_secs = secs;
        self
    }

    pub fn cleanup_interval(&self) -> u64 {
        self.cleanup_interval_secs
    }

    pub async fn revoke(&self, jti: &str, expires_at: DateTime<Utc>) {
        let mut blacklist = self.store.write().await;
        blacklist.insert(jti.to_string(), expires_at);
    }

    pub async fn is_revoked(&self, jti: &str) -> bool {
        let blacklist = self.store.read().await;
        if let Some(expires_at) = blacklist.get(jti) {
            Utc::now() < *expires_at
        } else {
            false
        }
    }

    pub async fn cleanup(&self) {
        let mut blacklist = self.store.write().await;
        let now = Utc::now();
        blacklist.retain(|_, exp| now < *exp);
    }

    pub async fn len(&self) -> usize {
        let blacklist = self.store.read().await;
        blacklist.len()
    }

    pub async fn is_empty(&self) -> bool {
        let blacklist = self.store.read().await;
        blacklist.is_empty()
    }
}

impl Default for TokenBlacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TokenBlacklist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenBlacklist")
            .field("cleanup_interval_secs", &self.cleanup_interval_secs)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_revoke_token() {
        let blacklist = TokenBlacklist::new();
        let expires = Utc::now() + Duration::hours(1);

        blacklist.revoke("test-jti", expires).await;
        assert!(blacklist.is_revoked("test-jti").await);
    }

    #[tokio::test]
    async fn test_not_revoked() {
        let blacklist = TokenBlacklist::new();
        assert!(!blacklist.is_revoked("unknown-jti").await);
    }

    #[tokio::test]
    async fn test_expired_revival() {
        let blacklist = TokenBlacklist::new();
        let expires = Utc::now() + Duration::seconds(-1);

        blacklist.revoke("expired-jti", expires).await;
        assert!(!blacklist.is_revoked("expired-jti").await);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let blacklist = TokenBlacklist::new();
        let expired = Utc::now() + Duration::seconds(-1);
        let valid = Utc::now() + Duration::hours(1);

        blacklist.revoke("expired-jti", expired).await;
        blacklist.revoke("valid-jti", valid).await;

        blacklist.cleanup().await;

        assert!(!blacklist.is_revoked("expired-jti").await);
        assert!(blacklist.is_revoked("valid-jti").await);
    }

    #[tokio::test]
    async fn test_len() {
        let blacklist = TokenBlacklist::new();
        let expires = Utc::now() + Duration::hours(1);

        assert_eq!(blacklist.len().await, 0);

        blacklist.revoke("jti-1", expires).await;
        assert_eq!(blacklist.len().await, 1);

        blacklist.revoke("jti-2", expires).await;
        assert_eq!(blacklist.len().await, 2);
    }

    #[tokio::test]
    async fn test_is_empty() {
        let blacklist = TokenBlacklist::new();
        assert!(blacklist.is_empty().await);

        let expires = Utc::now() + Duration::hours(1);
        blacklist.revoke("jti-1", expires).await;
        assert!(!blacklist.is_empty().await);
    }
}
