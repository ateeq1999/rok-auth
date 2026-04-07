//! Authorization audit logging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: AuditLevel,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub result: AuditResult,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    Login,
    Logout,
    LoginFailed,
    TokenRefresh,
    TokenRevoked,
    RoleAssigned,
    RoleRevoked,
    PermissionGranted,
    PermissionDenied,
    ResourceAccessed,
    ResourceModified,
    ResourceDeleted,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorFailed,
    OAuthConnected,
    OAuthDisconnected,
    SessionCreated,
    SessionRevoked,
    SessionExpired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
    Error,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType, result: AuditResult) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: AuditLevel::Info,
            event_type,
            user_id: None,
            resource: None,
            action: None,
            result,
            metadata: serde_json::json!({}),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.into(), value.into());
        }
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.level = level;
        self
    }
}

pub trait AuditLogger: Send + Sync {
    fn log(&self, event: &AuditEvent);
    fn get_events(&self, filter: &AuditFilter) -> Vec<AuditEvent>;
}

#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub user_id: Option<String>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub level: Option<AuditLevel>,
    pub result: Option<AuditResult>,
}

pub struct InMemoryAuditLogger {
    events: std::sync::Arc<tokio::sync::RwLock<VecDeque<AuditEvent>>>,
    max_events: usize,
}

impl InMemoryAuditLogger {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: std::sync::Arc::new(tokio::sync::RwLock::new(VecDeque::new())),
            max_events,
        }
    }

    pub async fn log(&self, event: AuditEvent) {
        let mut events = self.events.write().await;
        if events.len() >= self.max_events {
            events.pop_front();
        }
        events.push_back(event);
    }

    pub async fn get_events(&self, filter: &AuditFilter) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| self.matches_filter(e, filter))
            .cloned()
            .collect()
    }

    fn matches_filter(&self, event: &AuditEvent, filter: &AuditFilter) -> bool {
        if let Some(ref uid) = filter.user_id {
            if event.user_id.as_ref() != Some(uid) {
                return false;
            }
        }
        if let Some(ref types) = filter.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }
        if let Some(ref from) = filter.from_date {
            if event.timestamp < *from {
                return false;
            }
        }
        if let Some(ref to) = filter.to_date {
            if event.timestamp > *to {
                return false;
            }
        }
        if let Some(ref level) = filter.level {
            if event.level != *level {
                return false;
            }
        }
        if let Some(ref result) = filter.result {
            if event.result != *result {
                return false;
            }
        }
        true
    }
}

impl AuditLogger for InMemoryAuditLogger {
    fn log(&self, event: &AuditEvent) {
        let events = std::sync::Arc::clone(&self.events);
        let event = event.clone();
        tokio::spawn(async move {
            let mut events = events.write().await;
            if events.len() >= 10000 {
                events.pop_front();
            }
            events.push_back(event);
        });
    }

    fn get_events(&self, filter: &AuditFilter) -> Vec<AuditEvent> {
        let events = self.events.blocking_read();
        events.iter()
            .filter(|e| self.matches_filter(e, filter))
            .cloned()
            .collect()
    }
}

pub struct ConsoleAuditLogger;

impl AuditLogger for ConsoleAuditLogger {
    fn log(&self, event: &AuditEvent) {
        println!(
            "[{}] {:?} - {:?} - User: {:?} - Resource: {:?} - Result: {:?}",
            event.timestamp,
            event.level,
            event.event_type,
            event.user_id,
            event.resource,
            event.result
        );
    }

    fn get_events(&self, _filter: &AuditFilter) -> Vec<AuditEvent> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_creation() {
        let event = AuditEvent::new(AuditEventType::Login, AuditResult::Success)
            .with_user("user-123")
            .with_ip("192.168.1.1")
            .with_metadata("browser", "Chrome");

        assert!(event.user_id.is_some());
        assert!(event.ip_address.is_some());
    }

    #[tokio::test]
    async fn in_memory_logger() {
        let logger = InMemoryAuditLogger::new(100);
        
        logger.log(AuditEvent::new(AuditEventType::Login, AuditResult::Success)
            .with_user("user-1")).await;
        logger.log(AuditEvent::new(AuditEventType::Login, AuditResult::Failure)
            .with_user("user-2")).await;

        let events = logger.get_events(&AuditFilter::default()).await;
        assert_eq!(events.len(), 2);

        let filter = AuditFilter {
            user_id: Some("user-1".to_string()),
            ..Default::default()
        };
        let filtered = logger.get_events(&filter).await;
        assert_eq!(filtered.len(), 1);
    }
}
