//! Password hashing and verification using Argon2id.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::AuthError;

pub fn hash(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::HashError(e.to_string()))
}

pub fn verify(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|e| AuthError::HashError(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub async fn hash_async(password: String) -> Result<String, AuthError> {
    tokio::task::spawn_blocking(move || hash(&password))
        .await
        .map_err(|e| AuthError::HashError(e.to_string()))?
}

pub async fn verify_async(password: String, hash: String) -> Result<bool, AuthError> {
    tokio::task::spawn_blocking(move || verify(&password, &hash))
        .await
        .map_err(|e| AuthError::HashError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let h = hash("correct-horse-battery-staple").unwrap();
        assert!(verify("correct-horse-battery-staple", &h).unwrap());
        assert!(!verify("wrong-password", &h).unwrap());
    }

    #[test]
    fn hashes_are_unique() {
        let h1 = hash("same").unwrap();
        let h2 = hash("same").unwrap();
        assert_ne!(h1, h2);
    }
}
