//! Security event webhooks.
//!
//! Notifies external services of security events.

use chrono::{DateTime, Utc};
use serde::Serialize;

pub struct SecurityWebhook {
    url: String,
    secret: String,
    events: Vec<SecurityEventType>,
    client: Option<reqwest::Client>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SecurityEventType {
    Login,
    Logout,
    TokenRefresh,
    PasswordChange,
    PasswordReset,
    MfaEnabled,
    MfaDisabled,
    MfaChallenge,
    TokenRevoked,
    DeviceRegistered,
    DeviceRevoked,
    SuspiciousActivity,
}

impl SecurityWebhook {
    pub fn new(url: String, secret: String) -> Self {
        Self {
            url,
            secret,
            events: vec![
                SecurityEventType::PasswordChange,
                SecurityEventType::MfaEnabled,
                SecurityEventType::DeviceRegistered,
                SecurityEventType::SuspiciousActivity,
            ],
            client: None,
        }
    }

    pub fn with_events(mut self, events: Vec<SecurityEventType>) -> Self {
        self.events = events;
        self
    }

    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn subscribe(&mut self, event: SecurityEventType) {
        if !self.events.contains(&event) {
            self.events.push(event);
        }
    }

    pub fn unsubscribe(&mut self, event: SecurityEventType) {
        self.events.retain(|e| *e != event);
    }

    pub async fn send(&self, event: &SecurityAuditEvent) -> Result<(), WebhookError> {
        if !self.events.contains(&event.event_type) {
            return Ok(());
        }

        let payload = serde_json::to_vec(event)
            .map_err(|e| WebhookError::Serialization(e.to_string()))?;

        let signature = compute_signature(&payload, &self.secret);

        let client = self.client.clone().unwrap_or_else(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client")
        });

        client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("X-Security-Signature", signature)
            .header("X-Security-Event", event.event_type.to_string())
            .body(payload)
            .send()
            .await
            .map_err(WebhookError::Http)?;
        
        Ok(())
    }

    pub fn subscribed_events(&self) -> &[SecurityEventType] {
        &self.events
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityAuditEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub details: serde_json::Value,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::Login => write!(f, "login"),
            SecurityEventType::Logout => write!(f, "logout"),
            SecurityEventType::TokenRefresh => write!(f, "token_refresh"),
            SecurityEventType::PasswordChange => write!(f, "password_change"),
            SecurityEventType::PasswordReset => write!(f, "password_reset"),
            SecurityEventType::MfaEnabled => write!(f, "mfa_enabled"),
            SecurityEventType::MfaDisabled => write!(f, "mfa_disabled"),
            SecurityEventType::MfaChallenge => write!(f, "mfa_challenge"),
            SecurityEventType::TokenRevoked => write!(f, "token_revoked"),
            SecurityEventType::DeviceRegistered => write!(f, "device_registered"),
            SecurityEventType::DeviceRevoked => write!(f, "device_revoked"),
            SecurityEventType::SuspiciousActivity => write!(f, "suspicious_activity"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

fn compute_signature(payload: &[u8], secret: &str) -> String {
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(secret.as_bytes());
    hasher.update(payload);
    let result = hasher.finalize();
    hex::encode(result)
}

impl SecurityAuditEvent {
    pub fn login(user_id: &str, ip: Option<&str>, success: bool) -> Self {
        Self {
            event_type: SecurityEventType::Login,
            user_id: Some(user_id.to_string()),
            ip_address: ip.map(String::from),
            user_agent: None,
            timestamp: Utc::now(),
            success,
            details: serde_json::json!({}),
            risk_level: if success { RiskLevel::Low } else { RiskLevel::Medium },
        }
    }

    pub fn mfa_challenge(user_id: &str, success: bool) -> Self {
        Self {
            event_type: SecurityEventType::MfaChallenge,
            user_id: Some(user_id.to_string()),
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
            success,
            details: serde_json::json!({}),
            risk_level: if success { RiskLevel::Low } else { RiskLevel::High },
        }
    }

    pub fn suspicious_activity(user_id: Option<&str>, ip: Option<&str>, details: serde_json::Value) -> Self {
        Self {
            event_type: SecurityEventType::SuspiciousActivity,
            user_id: user_id.map(String::from),
            ip_address: ip.map(String::from),
            user_agent: None,
            timestamp: Utc::now(),
            success: false,
            details,
            risk_level: RiskLevel::High,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_display() {
        assert_eq!(SecurityEventType::Login.to_string(), "login");
        assert_eq!(SecurityEventType::MfaEnabled.to_string(), "mfa_enabled");
    }

    #[test]
    fn test_webhook_subscribe() {
        let mut webhook = SecurityWebhook::new(
            "https://example.com/webhook".to_string(),
            "secret".to_string(),
        );
        
        webhook.subscribe(SecurityEventType::Login);
        assert!(webhook.subscribed_events().contains(&SecurityEventType::Login));
        
        webhook.unsubscribe(SecurityEventType::Login);
        assert!(!webhook.subscribed_events().contains(&SecurityEventType::Login));
    }

    #[test]
    fn test_webhook_default_events() {
        let webhook = SecurityWebhook::new(
            "https://example.com/webhook".to_string(),
            "secret".to_string(),
        );
        
        assert!(webhook.subscribed_events().contains(&SecurityEventType::PasswordChange));
        assert!(webhook.subscribed_events().contains(&SecurityEventType::MfaEnabled));
    }

    #[test]
    fn test_compute_signature() {
        let payload = b"test payload";
        let signature = compute_signature(payload, "secret");
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_security_audit_event_login() {
        let event = SecurityAuditEvent::login("user-123", Some("192.168.1.1"), true);
        assert_eq!(event.event_type, SecurityEventType::Login);
        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert!(event.success);
        assert_eq!(event.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_security_audit_event_suspicious() {
        let details = serde_json::json!({"reason": "multiple_failed_logins", "attempts": 5});
        let event = SecurityAuditEvent::suspicious_activity(Some("user-123"), Some("192.168.1.1"), details);
        assert_eq!(event.event_type, SecurityEventType::SuspiciousActivity);
        assert_eq!(event.risk_level, RiskLevel::High);
    }
}