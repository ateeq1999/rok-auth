//! Rate limiting implementation using token bucket and sliding window algorithms.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_secs: u64,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_secs: 60,
            burst_size: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    Allowed,
    RateLimited { retry_after_secs: u64 },
}

pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<tokio::sync::RwLock<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    sliding_window: Vec<Instant>,
}

impl TokenBucket {
    fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Instant::now(),
            sliding_window: Vec::new(),
        }
    }

    fn try_consume(&mut self, config: &RateLimitConfig) -> RateLimitResult {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;

        self.tokens = (self.tokens + elapsed.as_secs_f64() * config.requests_per_window as f64
            / config.window_secs as f64)
            .min(config.burst_size as f64);

        let window_start = now - Duration::from_secs(config.window_secs);
        self.sliding_window.retain(|&t| t > window_start);

        if self.tokens >= 1.0 && self.sliding_window.len() < config.requests_per_window as usize {
            self.tokens -= 1.0;
            self.sliding_window.push(now);
            RateLimitResult::Allowed
        } else {
            let retry_after = if let Some(oldest) = self.sliding_window.first() {
                let next_available = *oldest + Duration::from_secs(config.window_secs);
                next_available.saturating_duration_since(now).as_secs()
            } else {
                config.window_secs
            };
            RateLimitResult::RateLimited {
                retry_after_secs: retry_after.max(1),
            }
        }
    }
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn check(&self, key: &str) -> RateLimitResult {
        let mut buckets = self.buckets.write().await;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(self.config.burst_size));
        bucket.try_consume(&self.config)
    }

    pub async fn check_user(&self, user_id: &str) -> RateLimitResult {
        self.check(&format!("user:{}", user_id)).await
    }

    pub async fn check_ip(&self, ip: &str) -> RateLimitResult {
        self.check(&format!("ip:{}", ip)).await
    }

    pub async fn check_endpoint(&self, ip: &str, endpoint: &str) -> RateLimitResult {
        self.check(&format!("{}:{}", ip, endpoint)).await
    }

    pub async fn cleanup(&self, max_entries: usize) {
        let mut buckets = self.buckets.write().await;
        if buckets.len() > max_entries {
            let to_remove = buckets.len() - max_entries;
            let keys: Vec<_> = buckets.keys().take(to_remove).cloned().collect();
            for key in keys {
                buckets.remove(&key);
            }
        }
    }

    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

pub struct MultiRateLimiter {
    limiters: HashMap<String, RateLimiter>,
    default_config: RateLimitConfig,
}

impl MultiRateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: HashMap::new(),
            default_config: RateLimitConfig::default(),
        }
    }

    pub fn with_config(name: &str, config: RateLimitConfig) -> Self {
        let mut limiters = HashMap::new();
        limiters.insert(name.to_string(), RateLimiter::new(config));
        Self {
            limiters,
            default_config: RateLimitConfig::default(),
        }
    }

    pub fn add_limiter(&mut self, name: &str, config: RateLimitConfig) {
        self.limiters.insert(name.to_string(), RateLimiter::new(config));
    }

    pub async fn check(&self, name: &str, key: &str) -> RateLimitResult {
        if let Some(limiter) = self.limiters.get(name) {
            limiter.check(key).await
        } else {
            RateLimitResult::Allowed
        }
    }

    pub async fn check_default(&self, key: &str) -> RateLimitResult {
        RateLimiter::new(self.default_config.clone())
            .check(key)
            .await
    }
}

impl Default for MultiRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rate_limiter_allows_under_limit() {
        let config = RateLimitConfig {
            requests_per_window: 10,
            window_secs: 60,
            burst_size: 10,
        };
        let limiter = RateLimiter::new(config);

        for _ in 0..5 {
            let result = limiter.check("test-key").await;
            assert_eq!(result, RateLimitResult::Allowed);
        }
    }

    #[tokio::test]
    async fn rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            requests_per_window: 3,
            window_secs: 60,
            burst_size: 3,
        };
        let limiter = RateLimiter::new(config);

        for _ in 0..3 {
            let _ = limiter.check("test-key").await;
        }

        let result = limiter.check("test-key").await;
        assert!(matches!(
            result,
            RateLimitResult::RateLimited {
                retry_after_secs: _
            }
        ));
    }

    #[test]
    fn rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_window, 100);
        assert_eq!(config.window_secs, 60);
        assert_eq!(config.burst_size, 10);
    }

    #[tokio::test]
    async fn different_keys_independent() {
        let limiter = RateLimiter::new(RateLimitConfig::default());

        for _ in 0..100 {
            let _ = limiter.check("key-1").await;
        }

        let result = limiter.check("key-2").await;
        assert_eq!(result, RateLimitResult::Allowed);
    }
}
