//! Email sender abstraction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub from: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

impl Email {
    pub fn new(to: Vec<String>, from: String, subject: String, html_body: String, text_body: String) -> Self {
        Self {
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            from,
            subject,
            html_body,
            text_body,
        }
    }

    pub fn simple(to: &str, from: &str, subject: &str, body: &str) -> Self {
        Self::new(
            vec![to.to_string()],
            from.to_string(),
            subject.to_string(),
            body.to_string(),
            body.to_string(),
        )
    }

    pub fn add_cc(&mut self, email: &str) {
        self.cc.push(email.to_string());
    }

    pub fn add_bcc(&mut self, email: &str) {
        self.bcc.push(email.to_string());
    }
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    pub fn new(host: String, port: u16, username: String, password: String, from_email: String, from_name: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            from_email,
            from_name,
            use_tls: true,
        }
    }

    pub fn from_url(url: &str) -> Result<Self, SendEmailError> {
        let parsed = url::Url::parse(url).map_err(|e| SendEmailError::InvalidUrl(e.to_string()))?;
        
        let host = parsed.host_str().ok_or_else(|| SendEmailError::InvalidUrl("no host".to_string()))?.to_string();
        let port = parsed.port().unwrap_or(587);
        let username = parsed.username().to_string();
        let password = parsed.password().unwrap_or("").to_string();
        let from_email = parsed.username().to_string();
        
        Ok(Self::new(host, port, username, password, from_email, "rok-auth".to_string()))
    }
}

pub trait EmailSender: Send + Sync {
    fn send(&self, email: &Email) -> impl std::future::Future<Output = Result<(), SendEmailError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum SendEmailError {
    #[error("failed to connect to SMTP server: {0}")]
    ConnectionFailed(String),
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("email rejected: {0}")]
    Rejected(String),
    #[error("timeout")]
    Timeout,
    #[error("invalid email address: {0}")]
    InvalidAddress(String),
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("send failed: {0}")]
    SendFailed(String),
}

pub struct ConsoleEmailSender;

impl EmailSender for ConsoleEmailSender {
    async fn send(&self, email: &Email) -> Result<(), SendEmailError> {
        println!("=== Email ===");
        println!("From: {}", email.from);
        println!("To: {:?}", email.to);
        if !email.cc.is_empty() {
            println!("CC: {:?}", email.cc);
        }
        println!("Subject: {}", email.subject);
        println!("--- HTML Body ---");
        println!("{}", email.html_body);
        println!("--- Text Body ---");
        println!("{}", email.text_body);
        println!("===============");
        Ok(())
    }
}

pub struct NoopEmailSender;

impl EmailSender for NoopEmailSender {
    async fn send(&self, _email: &Email) -> Result<(), SendEmailError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_email() {
        let email = Email::simple("test@example.com", "noreply@example.com", "Test", "Hello World");
        assert_eq!(email.to, vec!["test@example.com"]);
        assert_eq!(email.from, "noreply@example.com");
    }

    #[test]
    fn email_with_cc() {
        let mut email = Email::simple("to@example.com", "from@example.com", "Test", "Body");
        email.add_cc("cc@example.com");
        assert_eq!(email.cc, vec!["cc@example.com"]);
    }

    #[test]
    fn smtp_config_from_url() {
        let config = SmtpConfig::from_url("smtp://user:pass@smtp.example.com:587").unwrap();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.username, "user");
    }

    #[tokio::test]
    async fn console_sender_works() {
        let sender = ConsoleEmailSender;
        let email = Email::simple("to@example.com", "from@example.com", "Test", "Body");
        let result = sender.send(&email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn noop_sender_works() {
        let sender = NoopEmailSender;
        let email = Email::simple("to@example.com", "from@example.com", "Test", "Body");
        let result = sender.send(&email).await;
        assert!(result.is_ok());
    }
}
