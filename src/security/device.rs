//! Device tracking and session management.
//!
//! Provides device-based session tracking inspired by Laravel Sanctum.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Web,
    Mobile,
    Desktop,
    Api,
    Unknown,
}

impl DeviceType {
    pub fn from_user_agent(ua: &str) -> Self {
        let ua_lower = ua.to_lowercase();
        if ua_lower.contains("mobile") || ua_lower.contains("android") || ua_lower.contains("iphone") {
            DeviceType::Mobile
        } else if ua_lower.contains("curl") || ua_lower.contains("wget") || ua_lower.contains("postman") {
            DeviceType::Api
        } else if ua_lower.contains("windows") || ua_lower.contains("mac") || ua_lower.contains("linux") {
            DeviceType::Desktop
        } else {
            DeviceType::Web
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Unknown
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Web => write!(f, "web"),
            DeviceType::Mobile => write!(f, "mobile"),
            DeviceType::Desktop => write!(f, "desktop"),
            DeviceType::Api => write!(f, "api"),
            DeviceType::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub user_id: String,
    pub name: Option<String>,
    pub device_type: DeviceType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Device {
    pub fn new(user_id: String, device_type: DeviceType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            name: None,
            device_type,
            ip_address: None,
            user_agent: None,
            last_active: Utc::now(),
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_expiration(mut self, duration: chrono::Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        let ua_str = ua.into();
        self.user_agent = Some(ua_str.clone());
        self.device_type = DeviceType::from_user_agent(&ua_str);
        self
    }

    pub fn update_activity(&mut self) {
        self.last_active = Utc::now();
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }
}

pub struct DeviceManager {
    devices: Arc<RwLock<HashMap<String, Device>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, device: Device) -> String {
        let mut devices = self.devices.write().await;
        let id = device.id.clone();
        devices.insert(id.clone(), device);
        id
    }

    pub async fn get(&self, device_id: &str) -> Option<Device> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    pub async fn get_user_devices(&self, user_id: &str) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter(|d| d.user_id == user_id && !d.is_expired())
            .cloned()
            .collect()
    }

    pub async fn update_activity(&self, device_id: &str) -> bool {
        let mut devices = self.devices.write().await;
        if let Some(device) = devices.get_mut(device_id) {
            device.update_activity();
            true
        } else {
            false
        }
    }

    pub async fn revoke_device(&self, device_id: &str) -> bool {
        let mut devices = self.devices.write().await;
        devices.remove(device_id).is_some()
    }

    pub async fn revoke_all_user_devices(&self, user_id: &str) -> usize {
        let mut devices = self.devices.write().await;
        let initial_count = devices.len();
        devices.retain(|_, d| d.user_id != user_id);
        initial_count - devices.len()
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut devices = self.devices.write().await;
        let initial_count = devices.len();
        let now = Utc::now();
        devices.retain(|_, d| d.expires_at.map_or(true, |exp| now < exp));
        initial_count - devices.len()
    }

    pub async fn len(&self) -> usize {
        let devices = self.devices.read().await;
        devices.len()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_device() {
        let manager = DeviceManager::new();
        let device = Device::new("user-123".to_string(), DeviceType::Web)
            .with_name("Chrome on Windows");

        let id = manager.register(device).await;
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_get_device() {
        let manager = DeviceManager::new();
        let device = Device::new("user-123".to_string(), DeviceType::Web);

        let id = manager.register(device).await;
        let retrieved = manager.get(&id).await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_get_user_devices() {
        let manager = DeviceManager::new();

        manager
            .register(Device::new("user-1".to_string(), DeviceType::Web))
            .await;
        manager
            .register(Device::new("user-1".to_string(), DeviceType::Mobile))
            .await;
        manager
            .register(Device::new("user-2".to_string(), DeviceType::Web))
            .await;

        let user1_devices = manager.get_user_devices("user-1").await;
        assert_eq!(user1_devices.len(), 2);
    }

    #[tokio::test]
    async fn test_revoke_device() {
        let manager = DeviceManager::new();
        let id = manager
            .register(Device::new("user-1".to_string(), DeviceType::Web))
            .await;

        assert!(manager.revoke_device(&id).await);
        assert!(manager.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_revoke_all_user_devices() {
        let manager = DeviceManager::new();
        manager
            .register(Device::new("user-1".to_string(), DeviceType::Web))
            .await;
        manager
            .register(Device::new("user-1".to_string(), DeviceType::Mobile))
            .await;
        manager
            .register(Device::new("user-2".to_string(), DeviceType::Web))
            .await;

        let revoked = manager.revoke_all_user_devices("user-1").await;
        assert_eq!(revoked, 2);
    }

    #[test]
    fn test_device_type_from_user_agent() {
        assert_eq!(DeviceType::from_user_agent("Mozilla/5.0 (iPhone;)"), DeviceType::Mobile);
        assert_eq!(DeviceType::from_user_agent("Mozilla/5.0 (Windows NT 10.0;)"), DeviceType::Desktop);
        assert_eq!(DeviceType::from_user_agent("PostmanRuntime/7.28.0"), DeviceType::Api);
        assert_eq!(DeviceType::from_user_agent("Mozilla/5.0"), DeviceType::Web);
    }

    #[tokio::test]
    async fn test_device_expiration() {
        let device = Device::new("user-1".to_string(), DeviceType::Web)
            .with_expiration(chrono::Duration::seconds(-1));

        assert!(device.is_expired());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = DeviceManager::new();

        manager
            .register(
                Device::new("user-1".to_string(), DeviceType::Web)
                    .with_expiration(chrono::Duration::hours(1)),
            )
            .await;
        manager
            .register(
                Device::new("user-1".to_string(), DeviceType::Mobile)
                    .with_expiration(chrono::Duration::seconds(-1)),
            )
            .await;

        let cleaned = manager.cleanup_expired().await;
        assert_eq!(cleaned, 1);
    }
}