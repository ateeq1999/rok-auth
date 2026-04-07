//! Email templates for verification and password reset emails.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

impl EmailTemplate {
    pub fn verification_email(base_url: &str, token: &str, username: &str) -> Self {
        let verify_url = format!("{}/verify?token={}", base_url, token);
        Self {
            subject: "Verify your email address".to_string(),
            html_body: format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #2c3e50;">Verify your email</h1>
        <p>Hello {},</p>
        <p>Thank you for signing up! Please verify your email address by clicking the button below:</p>
        <p style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #3498db; color: white; padding: 12px 24px; 
               text-decoration: none; border-radius: 4px; display: inline-block;">Verify Email</a>
        </p>
        <p>Or copy and paste this link: <a href="{}">{}</a></p>
        <p style="color: #7f8c8d; font-size: 12px;">This link expires in 24 hours.</p>
    </div>
</body>
</html>"#,
                username, verify_url, verify_url, verify_url
            ),
            text_body: format!(
                "Hello {},\n\n\
                Thank you for signing up! Please verify your email address by visiting this link:\n\n\
                {}\n\n\
                This link expires in 24 hours.\n\n\
                If you didn't create an account, please ignore this email.",
                username, verify_url
            ),
        }
    }

    pub fn password_reset_email(base_url: &str, token: &str, username: &str) -> Self {
        let reset_url = format!("{}/reset-password?token={}", base_url, token);
        Self {
            subject: "Reset your password".to_string(),
            html_body: format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #e74c3c;">Password Reset Request</h1>
        <p>Hello {},</p>
        <p>We received a request to reset your password. Click the button below to set a new password:</p>
        <p style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #e74c3c; color: white; padding: 12px 24px; 
               text-decoration: none; border-radius: 4px; display: inline-block;">Reset Password</a>
        </p>
        <p>Or copy and paste this link: <a href="{}">{}</a></p>
        <p style="color: #7f8c8d; font-size: 12px;">This link expires in 1 hour. \
                If you didn't request this, please ignore this email.</p>
    </div>
</body>
</html>"#,
                username, reset_url, reset_url, reset_url
            ),
            text_body: format!(
                "Hello {},\n\n\
                We received a request to reset your password. Visit this link to set a new password:\n\n\
                {}\n\n\
                This link expires in 1 hour.\n\n\
                If you didn't request this, please ignore this email.",
                username, reset_url
            ),
        }
    }

    pub fn email_changed_email(new_email: &str) -> Self {
        Self {
            subject: "Your email has been changed".to_string(),
            html_body: format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h1 style="color: #27ae60;">Email Changed</h1>
        <p>Your email address has been successfully changed to: <strong>{}</strong></p>
        <p>If you did not make this change, please contact support immediately.</p>
    </div>
</body>
</html>"#,
                new_email
            ),
            text_body: format!(
                "Your email address has been successfully changed to: {}\n\n\
                If you did not make this change, please contact support immediately.",
                new_email
            ),
        }
    }
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render_verification(base_url: &str, token: &str, username: &str) -> EmailTemplate {
        EmailTemplate::verification_email(base_url, token, username)
    }

    pub fn render_password_reset(base_url: &str, token: &str, username: &str) -> EmailTemplate {
        EmailTemplate::password_reset_email(base_url, token, username)
    }

    pub fn render_email_changed(new_email: &str) -> EmailTemplate {
        EmailTemplate::email_changed_email(new_email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_email_contains_token() {
        let template =
            TemplateEngine::render_verification("https://example.com", "test-token-123", "John");
        assert!(template.html_body.contains("test-token-123"));
        assert!(template.html_body.contains("John"));
        assert!(template.subject.contains("Verify"));
    }

    #[test]
    fn password_reset_email_contains_token() {
        let template =
            TemplateEngine::render_password_reset("https://example.com", "reset-token-456", "Jane");
        assert!(template.html_body.contains("reset-token-456"));
        assert!(template.text_body.contains("reset-token-456"));
        assert!(template.subject.contains("Reset"));
    }

    #[test]
    fn email_changed_contains_new_email() {
        let template = TemplateEngine::render_email_changed("new@example.com");
        assert!(template.html_body.contains("new@example.com"));
        assert!(template.text_body.contains("new@example.com"));
    }
}
