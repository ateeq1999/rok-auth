//! Time-based One-Time Password (TOTP) authentication.

use hmac::Hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<sha1::Sha1>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    pub digits: u32,
    pub period_secs: u64,
    pub algorithm: TotpAlgorithm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TotpAlgorithm {
    Sha1,
    #[allow(dead_code)]
    Sha256,
    #[allow(dead_code)]
    Sha512,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            digits: 6,
            period_secs: 30,
            algorithm: TotpAlgorithm::Sha1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TotpCode(String);

impl TotpCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TotpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct TotpService {
    config: TotpConfig,
}

impl TotpService {
    pub fn new(config: TotpConfig) -> Self {
        Self { config }
    }

    pub fn generate_secret(&self) -> String {
        let mut bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes)
    }

    pub fn generate_code(&self, secret: &str) -> Result<TotpCode, TotpError> {
        let secret_bytes = decode_base32(secret)?;
        let time_step = current_time_step(self.config.period_secs);
        self.compute_code(&secret_bytes, time_step)
    }

    pub fn verify_code(&self, secret: &str, code: &str, tolerance: u32) -> Result<bool, TotpError> {
        let secret_bytes = decode_base32(secret)?;
        let time_step = current_time_step(self.config.period_secs);

        for offset in 0..=tolerance {
            let expected = self.compute_code(&secret_bytes, time_step - offset as u64)?;
            if constant_time_eq(expected.as_str(), code) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn compute_code(&self, secret: &[u8], time_step: u64) -> Result<TotpCode, TotpError> {
        use hmac::Mac;

        let counter_bytes = time_step.to_be_bytes();
        let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| TotpError::ComputationFailed)?;
        mac.update(&counter_bytes);
        let hmac = mac.finalize().into_bytes();
        let offset = (hmac[hmac.len() - 1] & 0x0f) as usize;

        let code = ((hmac[offset] & 0x7f) as u32) << 24
            | (hmac[offset + 1] as u32) << 16
            | (hmac[offset + 2] as u32) << 8
            | (hmac[offset + 3] as u32);

        let otp = code % 10u32.pow(self.config.digits);
        Ok(TotpCode(format!(
            "{:0>width$}",
            otp,
            width = self.config.digits as usize
        )))
    }

    pub fn provisioning_uri(&self, secret: &str, account_name: &str, issuer: &str) -> String {
        let label = format!(
            "{}:{}",
            urlencoding_encode(issuer),
            urlencoding_encode(account_name)
        );
        let params = format!(
            "secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
            secret,
            urlencoding_encode(issuer),
            self.config.digits,
            self.config.period_secs
        );
        format!("otpauth://totp/{}?{}", label, params)
    }
}

fn decode_base32(secret: &str) -> Result<Vec<u8>, TotpError> {
    base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret)
        .ok_or(TotpError::InvalidSecret)
}

fn current_time_step(period_secs: u64) -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / period_secs
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    #[error("invalid base32 secret")]
    InvalidSecret,
    #[error("TOTP computation failed")]
    ComputationFailed,
}

#[derive(Debug, Clone)]
pub struct BackupCode(String);

impl BackupCode {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut bytes);
        let code: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        Self(code)
    }

    pub fn verify(&self, input: &str) -> bool {
        constant_time_eq(&self.0, input)
    }
}

pub struct BackupCodes {
    codes: Vec<BackupCode>,
    used: usize,
}

impl BackupCodes {
    pub fn generate(count: usize) -> Self {
        Self {
            codes: (0..count).map(|_| BackupCode::generate()).collect(),
            used: 0,
        }
    }

    pub fn verify(&mut self, input: &str) -> bool {
        for (i, code) in self.codes.iter().enumerate() {
            if code.verify(input) && i >= self.used {
                self.used = i + 1;
                return true;
            }
        }
        false
    }

    pub fn remaining(&self) -> usize {
        self.codes.len() - self.used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> TotpService {
        TotpService::new(TotpConfig::default())
    }

    #[test]
    fn generate_secret() {
        let service = make_service();
        let secret = service.generate_secret();
        assert!(!secret.is_empty());
        assert!(secret.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_and_verify_code() {
        let service = make_service();
        let secret = service.generate_secret();
        let code = service.generate_code(&secret).unwrap();
        assert_eq!(code.as_str().len(), 6);
        assert!(code.as_str().chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn verify_correct_code() {
        let service = make_service();
        let secret = service.generate_secret();
        let code = service.generate_code(&secret).unwrap();
        let result = service.verify_code(&secret, code.as_str(), 1).unwrap();
        assert!(result);
    }

    #[test]
    fn reject_wrong_code() {
        let service = make_service();
        let secret = service.generate_secret();
        let result = service.verify_code(&secret, "000000", 1).unwrap();
        assert!(!result);
    }

    #[test]
    fn backup_codes_work() {
        let mut codes = BackupCodes::generate(10);
        let first = codes.codes[0].0.clone();
        assert!(codes.verify(&first));
        assert_eq!(codes.remaining(), 9);
        assert!(!codes.verify(&first));
    }
}
