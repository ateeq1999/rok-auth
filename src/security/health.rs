//! Health checks and metrics for rok-auth.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthState,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthMetrics {
    pub total_requests: u64,
    pub successful_auths: u64,
    pub failed_auths: u64,
    pub active_sessions: u64,
    pub tokens_issued: u64,
    pub tokens_revoked: u64,
    pub rate_limited_requests: u64,
    pub brute_force_attempts: u64,
}

#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<AuthMetrics>>,
    _start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(AuthMetrics::default())),
            _start_time: Instant::now(),
        }
    }

    pub async fn record_request(&self) {
        let mut m = self.metrics.write().await;
        m.total_requests += 1;
    }

    pub async fn record_successful_auth(&self) {
        let mut m = self.metrics.write().await;
        m.successful_auths += 1;
        m.tokens_issued += 1;
    }

    pub async fn record_failed_auth(&self) {
        let mut m = self.metrics.write().await;
        m.failed_auths += 1;
    }

    pub async fn record_session_created(&self) {
        let mut m = self.metrics.write().await;
        m.active_sessions += 1;
    }

    pub async fn record_session_revoked(&self) {
        let mut m = self.metrics.write().await;
        m.active_sessions = m.active_sessions.saturating_sub(1);
        m.tokens_revoked += 1;
    }

    pub async fn record_rate_limited(&self) {
        let mut m = self.metrics.write().await;
        m.rate_limited_requests += 1;
    }

    pub async fn record_brute_force_attempt(&self) {
        let mut m = self.metrics.write().await;
        m.brute_force_attempts += 1;
    }

    pub async fn get_metrics(&self) -> AuthMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn reset(&self) {
        let mut m = self.metrics.write().await;
        *m = AuthMetrics::default();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

type HealthCheckFn = dyn Fn() -> Pin<Box<dyn std::future::Future<Output = HealthCheck> + Send>> + Send + Sync;

pub struct HealthChecker {
    checks: HashMap<String, Box<HealthCheckFn>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }

    pub fn register<F, Fut>(&mut self, name: &str, check: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = HealthCheck> + Send + 'static,
    {
        self.checks.insert(
            name.to_string(),
            Box::new(move || Box::pin(check()) as Pin<Box<dyn std::future::Future<Output = HealthCheck> + Send>>),
        );
    }

    pub async fn check_all(&self) -> HealthStatus {
        let mut checks = Vec::new();
        let mut overall_state = HealthState::Healthy;

        for (_name, check_fn) in &self.checks {
            let result = check_fn().await;
            if result.status != HealthState::Healthy && overall_state == HealthState::Healthy {
                overall_state = result.status;
            } else if result.status == HealthState::Unhealthy {
                overall_state = HealthState::Unhealthy;
            }
            checks.push(result);
        }

        HealthStatus {
            status: overall_state,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: self.uptime().as_secs(),
            checks,
        }
    }

    pub fn uptime(&self) -> Duration {
        Duration::from_secs(0)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_health_check() -> HealthCheck {
    HealthCheck {
        name: "default".to_string(),
        status: HealthState::Healthy,
        latency_ms: Some(0),
        message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn metrics_collector_increments() {
        let collector = MetricsCollector::new();
        
        collector.record_request().await;
        collector.record_request().await;
        collector.record_successful_auth().await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_auths, 1);
    }

    #[tokio::test]
    async fn metrics_collector_sessions() {
        let collector = MetricsCollector::new();
        
        collector.record_session_created().await;
        collector.record_session_created().await;
        collector.record_session_revoked().await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.active_sessions, 1);
    }

    #[tokio::test]
    async fn health_checker_registers() {
        let mut checker = HealthChecker::new();
        checker.register("test", || async move {
            HealthCheck {
                name: "test".to_string(),
                status: HealthState::Healthy,
                latency_ms: Some(1),
                message: None,
            }
        });
        
        let status = checker.check_all().await;
        assert_eq!(status.status, HealthState::Healthy);
        assert_eq!(status.checks.len(), 1);
    }
}
