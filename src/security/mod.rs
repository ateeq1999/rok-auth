//! Security features for authentication.

pub mod blacklist;
pub mod cors;
pub mod csrf;
pub mod detection;
pub mod device;
pub mod headers;
pub mod health;
pub mod rate_limiter;
pub mod step_up;
pub mod webhook;

pub use blacklist::TokenBlacklist;
pub use cors::CorsConfig;
pub use csrf::CsrfProtection;
pub use detection::{
    BruteForceDetector, CredentialStuffingDetector, IpReputationChecker, StuffingReason,
    StuffingResult, SuspiciousActivity,
};
pub use device::{Device, DeviceManager, DeviceType};
pub use headers::SecurityHeaders;
pub use health::{
    AuthMetrics, HealthCheck, HealthChecker, HealthState, HealthStatus, MetricsCollector,
};
pub use rate_limiter::{MultiRateLimiter, RateLimitConfig, RateLimitResult, RateLimiter};
pub use step_up::StepUpAuth;
pub use webhook::{RiskLevel, SecurityAuditEvent, SecurityEventType, SecurityWebhook};
