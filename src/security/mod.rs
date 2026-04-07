//! Security features for authentication.

pub mod rate_limiter;
pub mod headers;
pub mod detection;
pub mod cors;
pub mod health;

pub use rate_limiter::{RateLimiter, RateLimitConfig, RateLimitResult, MultiRateLimiter};
pub use headers::SecurityHeaders;
pub use detection::{BruteForceDetector, SuspiciousActivity, IpReputationChecker};
pub use cors::CorsConfig;
pub use health::{HealthStatus, HealthState, HealthCheck, AuthMetrics, MetricsCollector, HealthChecker};
