//! CORS configuration for rok-auth.

use axum::http::HeaderValue;
use tower_http::cors::{AllowHeaders, CorsLayer, ExposeHeaders};

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_secs: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "X-Requested-With".to_string(),
            ],
            expose_headers: vec!["X-Request-Id".to_string()],
            allow_credentials: true,
            max_age_secs: 3600,
        }
    }
}

impl CorsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn permissive() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["*".to_string()],
            allow_headers: vec!["*".to_string()],
            expose_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age_secs: 86400,
        }
    }

    pub fn strict(origins: Vec<String>) -> Self {
        Self {
            allow_origins: origins,
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allow_headers: vec!["Authorization".to_string(), "Content-Type".to_string()],
            expose_headers: Vec::new(),
            allow_credentials: true,
            max_age_secs: 7200,
        }
    }

    pub fn to_layer(&self) -> CorsLayer {
        let mut layer = CorsLayer::new();

        if self.allow_origins.len() == 1 && self.allow_origins[0] == "*" {
            layer = layer.allow_origin(tower_http::cors::Any);
        } else {
            let origins: Vec<HeaderValue> = self
                .allow_origins
                .iter()
                .filter_map(|o| HeaderValue::from_str(o).ok())
                .collect();
            for origin in origins {
                layer = layer.allow_origin(origin);
            }
        }

        if self.allow_methods.len() == 1 && self.allow_methods[0] == "*" {
            layer = layer.allow_methods(tower_http::cors::Any);
        } else {
            let methods: Vec<_> = self
                .allow_methods
                .iter()
                .filter_map(|m| m.parse().ok())
                .collect();
            layer = layer.allow_methods(methods);
        }

        if self.allow_headers.len() == 1 && self.allow_headers[0] == "*" {
            layer = layer.allow_headers(tower_http::cors::Any);
        } else {
            let headers: Vec<_> = self
                .allow_headers
                .iter()
                .filter_map(|h| h.parse().ok())
                .collect();
            layer = layer.allow_headers(AllowHeaders::list(headers));
        }

        if !self.expose_headers.is_empty() {
            let expose: Vec<_> = self
                .expose_headers
                .iter()
                .filter_map(|h| h.parse().ok())
                .collect();
            layer = layer.expose_headers(ExposeHeaders::list(expose));
        }

        layer = layer.allow_credentials(self.allow_credentials);
        layer = layer.max_age(std::time::Duration::from_secs(self.max_age_secs));

        layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cors_config_default() {
        let config = CorsConfig::default();
        assert_eq!(config.allow_origins, vec!["*"]);
        assert!(config.allow_credentials);
    }

    #[test]
    fn cors_config_permissive() {
        let config = CorsConfig::permissive();
        assert_eq!(config.allow_origins, vec!["*"]);
        assert!(!config.allow_credentials);
    }

    #[test]
    fn cors_config_strict() {
        let origins = vec!["https://example.com".to_string()];
        let config = CorsConfig::strict(origins.clone());
        assert_eq!(config.allow_origins, origins);
        assert!(config.allow_credentials);
    }
}
