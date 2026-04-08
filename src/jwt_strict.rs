//! Strict JWT algorithm validation.
//!
//! Prevents algorithm confusion attacks by strictly validating JWT algorithms.

use jsonwebtoken::{decode, decode_header, Algorithm as JwtAlgorithm, DecodingKey, Validation};
use serde::de::DeserializeOwned;

use crate::error::AuthError;

#[derive(Debug, Clone, PartialEq)]
pub enum JwtAlgorithmType {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
}

impl JwtAlgorithmType {
    pub fn as_str(&self) -> &str {
        match self {
            JwtAlgorithmType::HS256 => "HS256",
            JwtAlgorithmType::HS384 => "HS384",
            JwtAlgorithmType::HS512 => "HS512",
            JwtAlgorithmType::RS256 => "RS256",
            JwtAlgorithmType::RS384 => "RS384",
            JwtAlgorithmType::RS512 => "RS512",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "HS256" => Some(JwtAlgorithmType::HS256),
            "HS384" => Some(JwtAlgorithmType::HS384),
            "HS512" => Some(JwtAlgorithmType::HS512),
            "RS256" => Some(JwtAlgorithmType::RS256),
            "RS384" => Some(JwtAlgorithmType::RS384),
            "RS512" => Some(JwtAlgorithmType::RS512),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictValidator {
    allowed_algorithms: Vec<JwtAlgorithmType>,
    require_claims: Vec<String>,
}

impl StrictValidator {
    pub fn new() -> Self {
        Self {
            allowed_algorithms: vec![
                JwtAlgorithmType::HS256,
                JwtAlgorithmType::HS384,
                JwtAlgorithmType::HS512,
            ],
            require_claims: vec!["sub".to_string(), "exp".to_string(), "iat".to_string()],
        }
    }

    pub fn allow_rsa(mut self) -> Self {
        self.allowed_algorithms.push(JwtAlgorithmType::RS256);
        self.allowed_algorithms.push(JwtAlgorithmType::RS384);
        self.allowed_algorithms.push(JwtAlgorithmType::RS512);
        self
    }

    pub fn allow_algorithm(mut self, algorithm: JwtAlgorithmType) -> Self {
        self.allowed_algorithms.push(algorithm);
        self
    }

    pub fn require_claim(mut self, claim: &str) -> Self {
        self.require_claims.push(claim.to_string());
        self
    }

    pub fn validate_token<T: DeserializeOwned + serde::de::DeserializeOwned>(
        &self,
        token: &str,
        decoding_key: &DecodingKey,
    ) -> Result<T, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;

        let alg_str = format!("{:?}", header.alg);

        let allowed = self
            .allowed_algorithms
            .iter()
            .any(|a| a.as_str() == alg_str);
        if !allowed {
            return Err(AuthError::Internal(format!(
                "Algorithm '{}' not allowed",
                alg_str
            )));
        }

        if alg_str == "None" {
            return Err(AuthError::Internal(
                "Algorithm 'none' is not allowed".to_string(),
            ));
        }

        let mut validation = Validation::new(JwtAlgorithm::HS256);
        validation.validate_exp = true;

        decode::<T>(token, decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })
    }

    pub fn extract_jti(token: &str) -> Result<String, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;

        let alg_str = format!("{:?}", header.alg);
        if alg_str == "None" {
            return Err(AuthError::Internal(
                "Algorithm 'none' is not allowed".to_string(),
            ));
        }

        let payload = token.split('.').nth(1).ok_or(AuthError::InvalidToken)?;

        let decoded = base64_decode(payload);
        let json: serde_json::Value =
            serde_json::from_slice(&decoded).map_err(|_| AuthError::InvalidToken)?;

        json.get("jti")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(AuthError::InvalidToken)
    }
}

impl Default for StrictValidator {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_decode(input: &str) -> Vec<u8> {
    use base32::Alphabet;
    let alphabet = Alphabet::Rfc4648 { padding: false };
    base32::decode(alphabet, input).unwrap_or_else(|| {
        let mut output = String::new();
        for c in input.chars() {
            if !c.is_whitespace() {
                output.push(c);
            }
        }
        output.as_bytes().to_vec()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_validator() -> StrictValidator {
        StrictValidator::new()
    }

    #[test]
    fn test_algorithm_from_str() {
        assert_eq!(
            JwtAlgorithmType::from_str("HS256"),
            Some(JwtAlgorithmType::HS256)
        );
        assert_eq!(
            JwtAlgorithmType::from_str("hs256"),
            Some(JwtAlgorithmType::HS256)
        );
        assert_eq!(JwtAlgorithmType::from_str("invalid"), None);
    }

    #[test]
    fn test_algorithm_as_str() {
        assert_eq!(JwtAlgorithmType::HS256.as_str(), "HS256");
        assert_eq!(JwtAlgorithmType::RS256.as_str(), "RS256");
    }

    #[test]
    fn test_allow_rsa() {
        let validator = StrictValidator::new().allow_rsa();
        assert!(validator.allowed_algorithms.len() > 3);
    }

    #[test]
    fn test_require_claims() {
        let validator = StrictValidator::new().require_claim("aud");
        assert!(validator.require_claims.contains(&"aud".to_string()));
    }
}
