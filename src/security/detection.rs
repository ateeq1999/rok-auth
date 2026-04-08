//! Brute force and suspicious activity detection.
//!
//! Includes detection for:
//! - Brute force attacks
//! - Credential stuffing
//! - IP reputation tracking

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
        attempts.get(identifier).map(|s| s.attempts).unwrap_or(0)
    }

    pub async fn unlock(&self, identifier: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(identifier);
    }

    pub async fn cleanup_expired(&self) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Instant::now();
        attempts.retain(|_, state| {
            if state.locked_until.is_some_and(|t| now < t) {
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

#[derive(Debug, Clone)]
pub struct CredentialStuffingDetector {
    ip_user_pairs: Arc<tokio::sync::RwLock<HashMap<String, Vec<CredentialPair>>>>,
    ip_password_pairs: Arc<tokio::sync::RwLock<HashMap<String, Vec<String>>>>,
    max_users_per_ip: usize,
    max_password_reuse: usize,
    window: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CredentialPair {
    user_id: String,
    password_hash: String,
    timestamp: Instant,
}

impl Default for CredentialStuffingDetector {
    fn default() -> Self {
        Self::new(5, 3, Duration::from_secs(3600))
    }
}

impl CredentialStuffingDetector {
    pub fn new(max_users_per_ip: usize, max_password_reuse: usize, window: Duration) -> Self {
        Self {
            ip_user_pairs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ip_password_pairs: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_users_per_ip,
            max_password_reuse,
            window,
        }
    }

    pub async fn check_login(
        &self,
        ip: &str,
        user_id: &str,
        password_hash: &str,
    ) -> StuffingResult {
        let now = Instant::now();
        let window_start = now - self.window;

        let ip_key = ip.to_string();

        {
            let pairs = self.ip_user_pairs.read().await;
            let user_count = pairs
                .get(&ip_key)
                .map(|v| v.iter().filter(|p| p.timestamp > window_start).count())
                .unwrap_or(0);

            if user_count >= self.max_users_per_ip {
                return StuffingResult::Suspicious {
                    reason: StuffingReason::TooManyUsersPerIp,
                    score: 50,
                };
            }
        }

        {
            let passwords = self.ip_password_pairs.read().await;
            let reuse_count = passwords
                .get(&ip_key)
                .map(|v| v.iter().filter(|p| *p == password_hash).count())
                .unwrap_or(0);

            if reuse_count >= self.max_password_reuse {
                return StuffingResult::Suspicious {
                    reason: StuffingReason::PasswordReuse,
                    score: 70,
                };
            }
        }

        {
            let mut pairs = self.ip_user_pairs.write().await;
            let entry = pairs.entry(ip_key.clone()).or_insert_with(Vec::new);
            entry.push(CredentialPair {
                user_id: user_id.to_string(),
                password_hash: password_hash.to_string(),
                timestamp: now,
            });
            entry.retain(|p| p.timestamp > window_start);
        }

        {
            let mut passwords = self.ip_password_pairs.write().await;
            let entry = passwords.entry(ip_key.clone()).or_insert_with(Vec::new);
            entry.push(password_hash.to_string());
            entry.sort();
            entry.dedup();
            if entry.len() > self.max_password_reuse {
                entry.truncate(self.max_password_reuse);
            }
        }

        StuffingResult::Allowed
    }

    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window_start = now - self.window;

        let mut pairs = self.ip_user_pairs.write().await;
        pairs.retain(|_, v| v.iter().any(|p| p.timestamp > window_start));

        let mut passwords = self.ip_password_pairs.write().await;
        passwords.retain(|_, v| !v.is_empty());
    }
}

#[derive(Debug, Clone)]
pub enum StuffingResult {
    Allowed,
    Suspicious { reason: StuffingReason, score: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StuffingReason {
    TooManyUsersPerIp,
    PasswordReuse,
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
        let detector =
            BruteForceDetector::new(3, Duration::from_secs(60), Duration::from_secs(300));

        for _ in 0..3 {
            let _ = detector.record_failed_attempt("test-user").await;
        }

        let result = detector.record_failed_attempt("test-user").await;
        assert!(matches!(
            result,
            AttemptResult::Locked { remaining_secs: _ }
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

        checker.report_bad_ip("1.2.3.4", -150, "brute_force").await;

        let result = checker.check_ip("1.2.3.4").await;
        assert!(result.blocked);
        assert!(result.suspicious);
        assert_eq!(result.score, -150);
    }

    #[tokio::test]
    async fn credential_stuffing_allows_normal_logins() {
        let detector = CredentialStuffingDetector::default();

        let result = detector.check_login("1.2.3.4", "user1", "hash1").await;
        assert!(matches!(result, StuffingResult::Allowed));

        let result = detector.check_login("1.2.3.4", "user2", "hash2").await;
        assert!(matches!(result, StuffingResult::Allowed));
    }

    #[tokio::test]
    async fn credential_stuffing_detects_many_users() {
        let detector = CredentialStuffingDetector::new(3, 10, Duration::from_secs(3600));

        for i in 0..5 {
            let result = detector
                .check_login("1.2.3.4", &format!("user{}", i), &format!("hash{}", i))
                .await;
            if i < 3 {
                assert!(matches!(result, StuffingResult::Allowed));
            } else {
                assert!(matches!(
                    result,
                    StuffingResult::Suspicious {
                        reason: StuffingReason::TooManyUsersPerIp,
                        ..
                    }
                ));
            }
        }
    }

    #[tokio::test]
    async fn credential_stuffing_tracks_unique_passwords() {
        let detector = CredentialStuffingDetector::new(3, 5, Duration::from_secs(3600));

        // Different passwords should all be allowed (within limit)
        for i in 0..3 {
            let result = detector
                .check_login("1.2.3.4", &format!("user{}", i), &format!("hash{}", i))
                .await;
            assert!(matches!(result, StuffingResult::Allowed));
        }

        // Too many unique passwords from same IP is suspicious
        let result = detector
            .check_login("1.2.3.4", "user_extra", "another_hash")
            .await;
        assert!(matches!(
            result,
            StuffingResult::Suspicious {
                reason: StuffingReason::TooManyUsersPerIp,
                ..
            }
        ));
    }
}
