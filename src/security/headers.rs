//! Security headers middleware for HTTP responses.

use axum::http::{HeaderMap, HeaderValue, Response};

#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    pub strict_transport_security: Option<String>,
    pub content_security_policy: Option<String>,
    pub x_content_type_options: Option<String>,
    pub x_frame_options: Option<String>,
    pub x_xss_protection: Option<String>,
    pub referrer_policy: Option<String>,
    pub permissions_policy: Option<String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            strict_transport_security: Some("max-age=31536000; includeSubDomains".to_string()),
            content_security_policy: Some(
                "default-src 'self'; script-src 'self'; object-src 'none'; frame-ancestors 'none'"
                    .to_string(),
            ),
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            x_xss_protection: Some("1; mode=block".to_string()),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some("geolocation=(), microphone=(), camera=()".to_string()),
        }
    }
}

impl SecurityHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hsts(mut self, max_age: u32, include_subdomains: bool) -> Self {
        let mut value = format!("max-age={}", max_age);
        if include_subdomains {
            value.push_str("; includeSubDomains");
        }
        value.push_str("; preload");
        self.strict_transport_security = Some(value);
        self
    }

    pub fn with_csp(mut self, policy: &str) -> Self {
        self.content_security_policy = Some(policy.to_string());
        self
    }

    pub fn strict(mut self) -> Self {
        self.strict_transport_security =
            Some("max-age=31536000; includeSubDomains; preload".to_string());
        self.content_security_policy = Some(
            "default-src 'none'; script-src 'none'; style-src 'nonce'; img-src 'self'; \
             form-action 'self'; frame-ancestors 'none'; base-uri 'self'"
                .to_string(),
        );
        self.x_frame_options = Some("DENY".to_string());
        self
    }

    pub fn permissive(mut self) -> Self {
        self.strict_transport_security = None;
        self.content_security_policy = None;
        self.x_frame_options = None;
        self
    }

    pub fn apply(&self, response: Response<&str>) -> Response<String> {
        let mut builder = Response::builder().status(response.status());

        let headers = builder.headers_mut().unwrap();

        if let Some(ref _hsts) = self.strict_transport_security {
            headers.insert(
                "Strict-Transport-Security",
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
        }

        if let Some(ref csp) = self.content_security_policy {
            if let Ok(val) = HeaderValue::from_str(csp) {
                headers.insert("Content-Security-Policy", val);
            }
        }

        if let Some(ref xcto) = self.x_content_type_options {
            if let Ok(val) = HeaderValue::from_str(xcto) {
                headers.insert("X-Content-Type-Options", val);
            }
        }

        if let Some(ref xfo) = self.x_frame_options {
            if let Ok(val) = HeaderValue::from_str(xfo) {
                headers.insert("X-Frame-Options", val);
            }
        }

        if let Some(ref xxss) = self.x_xss_protection {
            if let Ok(val) = HeaderValue::from_str(xxss) {
                headers.insert("X-XSS-Protection", val);
            }
        }

        if let Some(ref rp) = self.referrer_policy {
            if let Ok(val) = HeaderValue::from_str(rp) {
                headers.insert("Referrer-Policy", val);
            }
        }

        if let Some(ref pp) = self.permissions_policy {
            if let Ok(val) = HeaderValue::from_str(pp) {
                headers.insert("Permissions-Policy", val);
            }
        }

        for (name, value) in response.headers() {
            headers.insert(name, value.clone());
        }

        let body = response.body().to_string();
        builder.body(body).unwrap()
    }
}

pub fn default_headers() -> HeaderMap {
    let headers = SecurityHeaders::new();
    let mut map = HeaderMap::new();

    if let Some(hsts) = headers.strict_transport_security {
        map.insert("Strict-Transport-Security", hsts.parse().unwrap());
    }
    if let Some(csp) = headers.content_security_policy {
        map.insert("Content-Security-Policy", csp.parse().unwrap());
    }
    if let Some(xcto) = headers.x_content_type_options {
        map.insert("X-Content-Type-Options", xcto.parse().unwrap());
    }
    if let Some(xfo) = headers.x_frame_options {
        map.insert("X-Frame-Options", xfo.parse().unwrap());
    }
    if let Some(xxss) = headers.x_xss_protection {
        map.insert("X-XSS-Protection", xxss.parse().unwrap());
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_headers_default() {
        let headers = SecurityHeaders::new();
        assert!(headers.strict_transport_security.is_some());
        assert!(headers.content_security_policy.is_some());
        assert_eq!(headers.x_frame_options, Some("DENY".to_string()));
    }

    #[test]
    fn security_headers_strict() {
        let headers = SecurityHeaders::new().strict();
        assert!(headers.strict_transport_security.is_some());
        assert!(headers.content_security_policy.is_some());
    }

    #[test]
    fn security_headers_permissive() {
        let headers = SecurityHeaders::new().permissive();
        assert!(headers.strict_transport_security.is_none());
        assert!(headers.content_security_policy.is_none());
    }
}
