//! CORS origin validation and security policy guard for Kite x402 Axum routes.

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CorsGuardConfig {
    pub allowed_origins: HashSet<String>,
    pub allow_credentials: bool,
    pub max_age_secs: u64,
}

impl Default for CorsGuardConfig {
    fn default() -> Self {
        Self {
            allowed_origins: HashSet::new(),
            allow_credentials: true,
            max_age_secs: 86400,
        }
    }
}

#[derive(Debug)]
pub struct CorsSecurityGuard {
    config: CorsGuardConfig,
}

impl CorsSecurityGuard {
    pub fn new(allowed_origins: Vec<impl Into<String>>) -> Self {
        let mut origins = HashSet::new();
        for o in allowed_origins {
            origins.insert(o.into());
        }
        Self {
            config: CorsGuardConfig {
                allowed_origins: origins,
                ..Default::default()
            },
        }
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allowed_origins.contains("*") {
            return true;
        }
        self.config.allowed_origins.contains(origin)
    }

    pub fn get_cors_headers(&self, origin: &str) -> Option<Vec<(&'static str, String)>> {
        if !self.is_origin_allowed(origin) {
            return None;
        }

        Some(vec![
            ("Access-Control-Allow-Origin", origin.to_string()),
            (
                "Access-Control-Allow-Methods",
                "GET, POST, OPTIONS".to_string(),
            ),
            (
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, PAYMENT-SIGNATURE".to_string(),
            ),
            (
                "Access-Control-Expose-Headers",
                "PAYMENT-REQUIRED, PAYMENT-RESPONSE".to_string(),
            ),
            (
                "Access-Control-Max-Age",
                self.config.max_age_secs.to_string(),
            ),
        ])
    }
}
