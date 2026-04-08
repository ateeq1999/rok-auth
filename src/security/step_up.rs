//! Step-up authentication for sensitive operations.
//!
//! Requires fresh authentication for sensitive operations.

use chrono::{DateTime, Duration, Utc};

pub struct StepUpAuth {
    freshness_duration: Duration,
}

impl StepUpAuth {
    pub fn new() -> Self {
        Self {
            freshness_duration: Duration::seconds(300),
        }
    }

    pub fn with_duration(mut self, secs: u64) -> Self {
        self.freshness_duration = Duration::seconds(secs as i64);
        self
    }

    pub fn freshness_duration(&self) -> u64 {
        self.freshness_duration.num_seconds() as u64
    }

    pub fn requires_reauth(&self, issued_at: i64) -> bool {
        let issued_at_dt = DateTime::from_timestamp(issued_at, 0).unwrap_or_else(|| Utc::now());
        Utc::now() - issued_at_dt > self.freshness_duration
    }

    pub fn is_fresh(&self, issued_at: i64) -> bool {
        !self.requires_reauth(issued_at)
    }
}

impl Default for StepUpAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StepUpAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StepUpAuth")
            .field(
                "freshness_duration_secs",
                &self.freshness_duration.num_seconds(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step_up() -> StepUpAuth {
        StepUpAuth::new()
    }

    #[test]
    fn test_default_5_minutes() {
        let step_up = StepUpAuth::new();
        assert_eq!(step_up.freshness_duration(), 300);
    }

    #[test]
    fn test_custom_duration() {
        let step_up = StepUpAuth::new().with_duration(600);
        assert_eq!(step_up.freshness_duration(), 600);
    }

    #[test]
    fn test_recent_token_is_fresh() {
        let step_up = StepUpAuth::new();
        let now = Utc::now().timestamp();
        assert!(!step_up.requires_reauth(now));
        assert!(step_up.is_fresh(now));
    }

    #[test]
    fn test_old_token_requires_reauth() {
        let step_up = StepUpAuth::new().with_duration(300);
        let old = (Utc::now() - Duration::seconds(400)).timestamp();
        assert!(step_up.requires_reauth(old));
        assert!(!step_up.is_fresh(old));
    }

    #[test]
    fn test_boundary_token() {
        let step_up = StepUpAuth::new().with_duration(300);
        let boundary = (Utc::now() - Duration::seconds(299)).timestamp();
        assert!(!step_up.requires_reauth(boundary));
        assert!(step_up.is_fresh(boundary));
    }
}
