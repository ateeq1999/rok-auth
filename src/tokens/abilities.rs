//! Token abilities and scopes module.
//!
//! Provides token-based authorization with abilities inspired by Laravel Sanctum.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenAbility {
    Read,
    Write,
    Delete,
    Manage,
    Custom(String),
}

impl TokenAbility {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "read" => TokenAbility::Read,
            "write" => TokenAbility::Write,
            "delete" => TokenAbility::Delete,
            "manage" => TokenAbility::Manage,
            other => TokenAbility::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TokenAbility::Read => "read",
            TokenAbility::Write => "write",
            TokenAbility::Delete => "delete",
            TokenAbility::Manage => "manage",
            TokenAbility::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for TokenAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenWithAbilities {
    pub token: String,
    pub abilities: Vec<TokenAbility>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub name: Option<String>,
}

impl TokenWithAbilities {
    pub fn new(token: String, abilities: Vec<TokenAbility>) -> Self {
        Self {
            token,
            abilities,
            expires_at: None,
            created_at: Utc::now(),
            name: None,
        }
    }

    pub fn with_expiration(mut self, duration: chrono::Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn can(&self, ability: &TokenAbility) -> bool {
        self.abilities.iter().any(|a| a == ability)
    }

    pub fn can_any(&self, abilities: &[TokenAbility]) -> bool {
        abilities.iter().any(|a| self.can(a))
    }

    pub fn can_all(&self, abilities: &[TokenAbility]) -> bool {
        abilities.iter().all(|a| self.can(a))
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_ability_from_str() {
        assert_eq!(TokenAbility::from_str("read"), TokenAbility::Read);
        assert_eq!(TokenAbility::from_str("WRITE"), TokenAbility::Write);
        assert_eq!(
            TokenAbility::from_str("custom:admin"),
            TokenAbility::Custom("custom:admin".to_string())
        );
    }

    #[test]
    fn test_token_ability_as_str() {
        assert_eq!(TokenAbility::Read.as_str(), "read");
        assert_eq!(TokenAbility::Write.as_str(), "write");
    }

    #[test]
    fn test_token_with_abilities_can() {
        let token = TokenWithAbilities::new(
            "test_token".to_string(),
            vec![TokenAbility::Read, TokenAbility::Write],
        );

        assert!(token.can(&TokenAbility::Read));
        assert!(token.can(&TokenAbility::Write));
        assert!(!token.can(&TokenAbility::Delete));
    }

    #[test]
    fn test_token_with_abilities_can_any() {
        let token = TokenWithAbilities::new("test_token".to_string(), vec![TokenAbility::Read]);

        assert!(token.can_any(&[TokenAbility::Read, TokenAbility::Write]));
        assert!(!token.can_any(&[TokenAbility::Delete, TokenAbility::Manage]));
    }

    #[test]
    fn test_token_with_abilities_can_all() {
        let token = TokenWithAbilities::new(
            "test_token".to_string(),
            vec![TokenAbility::Read, TokenAbility::Write],
        );

        assert!(token.can_all(&[TokenAbility::Read]));
        assert!(token.can_all(&[TokenAbility::Read, TokenAbility::Write]));
        assert!(!token.can_all(&[TokenAbility::Read, TokenAbility::Delete]));
    }

    #[test]
    fn test_token_with_abilities_expiration() {
        let token = TokenWithAbilities::new("test".to_string(), vec![])
            .with_expiration(chrono::Duration::hours(1));

        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_with_abilities_name() {
        let token = TokenWithAbilities::new("test".to_string(), vec![]).with_name("iPhone 12");

        assert_eq!(token.name, Some("iPhone 12".to_string()));
    }
}
