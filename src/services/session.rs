//! Session management service.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::session::SessionToken;
use crate::AuthError;

#[derive(Clone)]
pub struct SessionData {
    pub user_id: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct SessionService {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    ttl_seconds: u64,
}

impl SessionService {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            ttl_seconds,
        }
    }

    pub async fn create_session(
        &self,
        user_id: String,
        roles: Vec<String>,
    ) -> (SessionToken, SessionData) {
        let token = SessionToken::generate();
        let data = SessionData {
            user_id: user_id.clone(),
            roles: roles.clone(),
            created_at: chrono::Utc::now(),
        };
        let mut sessions = self.sessions.write().await;
        sessions.insert(token.as_str().to_string(), data.clone());
        (token, data)
    }

    pub async fn get_session(&self, token: &str) -> Result<SessionData, AuthError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(token)
            .cloned()
            .ok_or(AuthError::InvalidToken)
    }

    pub async fn delete_session(&self, token: &str) -> Result<(), AuthError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(token);
        Ok(())
    }

    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        let now = chrono::Utc::now().timestamp() as u64;
        sessions.retain(|_, data| {
            let created = data.created_at.timestamp() as u64;
            now - created < self.ttl_seconds
        });
    }
}
