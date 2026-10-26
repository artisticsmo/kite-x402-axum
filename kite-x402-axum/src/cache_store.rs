//! In-memory TTL payment verification cache store for Kite x402 services.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CachedVerification {
    pub is_valid: bool,
    pub payer: String,
    pub amount: String,
    pub cached_at: Instant,
    pub ttl: Duration,
}

#[derive(Debug)]
pub struct VerificationCacheStore {
    default_ttl: Duration,
    entries: Mutex<HashMap<String, CachedVerification>>,
}

impl VerificationCacheStore {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            default_ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, signature_hash: &str) -> Option<CachedVerification> {
        let mut map = self.entries.lock().unwrap();
        if let Some(entry) = map.get(signature_hash) {
            if entry.cached_at.elapsed() < entry.ttl {
                return Some(entry.clone());
            } else {
                map.remove(signature_hash);
            }
        }
        None
    }

    pub fn insert(&self, signature_hash: &str, is_valid: bool, payer: &str, amount: &str, custom_ttl: Option<Duration>) {
        let mut map = self.entries.lock().unwrap();
        let ttl = custom_ttl.unwrap_or(self.default_ttl);
        map.insert(
            signature_hash.to_string(),
            CachedVerification {
                is_valid,
                payer: payer.to_string(),
                amount: amount.to_string(),
                cached_at: Instant::now(),
                ttl,
            },
        );
    }

    pub fn clear(&self) {
        let mut map = self.entries.lock().unwrap();
        map.clear();
    }
}
