//! Security features for authentication.

pub mod rate_limiter;
pub mod headers;
pub mod detection;

pub use rate_limiter::{RateLimiter, RateLimitConfig, RateLimitResult};
pub use headers::SecurityHeaders;
pub use detection::{BruteForceDetector, SuspiciousActivity};
