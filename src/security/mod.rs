//! Security features for authentication.

pub mod rate_limiter;
pub mod headers;
pub mod detection;
pub mod cors;
pub mod health;
pub mod blacklist;
pub mod device;
pub mod csrf;
pub mod step_up;
pub mod webhook;

pub use rate_limiter::{RateLimiter, RateLimitConfig, RateLimitResult, MultiRateLimiter};
pub use headers::SecurityHeaders;
pub use detection::{BruteForceDetector, SuspiciousActivity, IpReputationChecker, CredentialStuffingDetector, StuffingResult, StuffingReason};
pub use cors::CorsConfig;
pub use health::{HealthStatus, HealthState, HealthCheck, AuthMetrics, MetricsCollector, HealthChecker};
pub use blacklist::TokenBlacklist;
pub use device::{Device, DeviceManager, DeviceType};
pub use csrf::CsrfProtection;
pub use step_up::StepUpAuth;
pub use webhook::{SecurityWebhook, SecurityAuditEvent, SecurityEventType, RiskLevel};
