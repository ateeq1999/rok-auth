//! Brute force and suspicious activity detection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SuspiciousActivity {
    pub activity_type: ActivityType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: Instant,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActivityType {
    FailedLogin,
    PasswordReset,
    AccountLockout,
    SuspiciousIp,
    RateLimitExceeded,
    InvalidTotp,
}

impl SuspiciousActivity {
    pub fn new(activity_type: ActivityType, details: &str) -> Self {
        Self {
            activity_type,
            user_id: None,
            ip_address: None,
            timestamp: Instant::now(),
            details: details.to_string(),
        }
    }

    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct BruteForceDetector {
    failed_attempts: Arc<tokio::sync::RwLock<HashMap<String, FailedAttemptState>>>,
    max_attempts: u32,
    lockout_duration: Duration,
    detection_window: Duration,
}

#[derive(Debug)]
struct FailedAttemptState {
    attempts: u32,
    first_attempt: Instant,
    locked_until: Option<Instant>,
}

impl Default for BruteForceDetector {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(900), Duration::from_secs(300))
    }
}

impl BruteForceDetector {
    pub fn new(max_attempts: u32, lockout_duration: Duration, detection_window: Duration) -> Self {
        Self {
            failed_attempts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_attempts,
            lockout_duration,
            detection_window,
        }
    }

    pub async fn record_failed_attempt(&self, identifier: &str) -> AttemptResult {
        let mut attempts = self.failed_attempts.write().await;
        let state = attempts
            .entry(identifier.to_string())
            .or_insert_with(|| FailedAttemptState {
                attempts: 0,
                first_attempt: Instant::now(),
                locked_until: None,
            });

        let now = Instant::now();

        if let Some(locked_until) = state.locked_until {
            if now < locked_until {
                return AttemptResult::Locked {
                    remaining_secs: locked_until.duration_since(now).as_secs(),
                };
            }
            state.attempts = 0;
            state.locked_until = None;
        }

        if now.duration_since(state.first_attempt) > self.detection_window {
            state.attempts = 1;
            state.first_attempt = now;
        } else {
            state.attempts += 1;
        }

        if state.attempts >= self.max_attempts {
            state.locked_until = Some(now + self.lockout_duration);
            return AttemptResult::Locked {
                remaining_secs: self.lockout_duration.as_secs(),
            };
        }

        AttemptResult::Allowed {
            attempts_remaining: self.max_attempts - state.attempts,
        }
    }

    pub async fn record_successful_attempt(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(identifier);
    }

    pub async fn is_locked(&self, identifier: &str) -> bool {
        let attempts = self.failed_attempts.read().await;
        if let Some(state) = attempts.get(identifier) {
            if let Some(locked_until) = state.locked_until {
                return Instant::now() < locked_until;
            }
        }
        false
    }

    pub async fn get_failed_attempts(&self, identifier: &str) -> u32 {
        let attempts = self.failed_attempts.read().await;
        attempts
            .get(identifier)
            .map(|s| s.attempts)
            .unwrap_or(0)
    }

    pub async fn unlock(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(identifier);
    }

    pub async fn cleanup_expired(&self) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Instant::now();
        attempts.retain(|_, state| {
            if state.locked_until.map_or(false, |t| now < t) {
                return true;
            }
            now.duration_since(state.first_attempt) <= self.detection_window
        });
    }
}

#[derive(Debug, Clone)]
pub enum AttemptResult {
    Allowed { attempts_remaining: u32 },
    Locked { remaining_secs: u64 },
}

pub struct IpReputationChecker {
    blocked_ips: Arc<tokio::sync::RwLock<HashMap<String, IpReputation>>>,
}

#[derive(Debug, Clone)]
struct IpReputation {
    score: i32,
    last_seen: Instant,
    flags: Vec<String>,
}

impl Default for IpReputationChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl IpReputationChecker {
    pub fn new() -> Self {
        Self {
            blocked_ips: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_ip(&self, ip: &str) -> IpReputationResult {
        let ips = self.blocked_ips.read().await;
        if let Some(reputation) = ips.get(ip) {
            let score = reputation.score;
            let blocked = score <= -100;
            let suspicious = score < 0;
            IpReputationResult {
                blocked,
                suspicious,
                score,
                flags: reputation.flags.clone(),
            }
        } else {
            IpReputationResult {
                blocked: false,
                suspicious: false,
                score: 0,
                flags: Vec::new(),
            }
        }
    }

    pub async fn report_bad_ip(&self, ip: &str, delta: i32, reason: &str) {
        let mut ips = self.blocked_ips.write().await;
        let entry = ips.entry(ip.to_string()).or_insert_with(|| IpReputation {
            score: 0,
            last_seen: Instant::now(),
            flags: Vec::new(),
        });
        entry.score += delta;
        entry.last_seen = Instant::now();
        if !entry.flags.iter().any(|f| f == reason) {
            entry.flags.push(reason.to_string());
        }
    }

    pub async fn whitelist_ip(&self, ip: &str) {
        let mut ips = self.blocked_ips.write().await;
        ips.remove(ip);
    }

    pub async fn cleanup_stale(&self, max_age: Duration) {
        let mut ips = self.blocked_ips.write().await;
        let now = Instant::now();
        ips.retain(|_, rep| now.duration_since(rep.last_seen) < max_age);
    }
}

#[derive(Debug, Clone)]
pub struct IpReputationResult {
    pub blocked: bool,
    pub suspicious: bool,
    pub score: i32,
    pub flags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn brute_force_detector_allows_under_limit() {
        let detector = BruteForceDetector::default();

        for _ in 0..4 {
            let result = detector.record_failed_attempt("test-user").await;
            assert!(matches!(
                result,
                AttemptResult::Allowed {
                    attempts_remaining: _
                }
            ));
        }
    }

    #[tokio::test]
    async fn brute_force_detector_locks_after_max_attempts() {
        let detector = BruteForceDetector::new(3, Duration::from_secs(60), Duration::from_secs(300));

        for _ in 0..3 {
            let _ = detector.record_failed_attempt("test-user").await;
        }

        let result = detector.record_failed_attempt("test-user").await;
        assert!(matches!(
            result,
            AttemptResult::Locked {
                remaining_secs: _
            }
        ));
    }

    #[tokio::test]
    async fn successful_attempt_resets_counter() {
        let detector = BruteForceDetector::default();

        for _ in 0..3 {
            let _ = detector.record_failed_attempt("test-user").await;
        }

        detector.record_successful_attempt("test-user").await;

        let attempts = detector.get_failed_attempts("test-user").await;
        assert_eq!(attempts, 0);
    }

    #[tokio::test]
    async fn ip_reputation_blocks_bad_ips() {
        let checker = IpReputationChecker::new();

        checker
            .report_bad_ip("1.2.3.4", -150, "brute_force")
            .await;

        let result = checker.check_ip("1.2.3.4").await;
        assert!(result.blocked);
        assert!(result.suspicious);
        assert_eq!(result.score, -150);
    }
}
