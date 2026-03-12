use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::FromRef;
use rumqttc::AsyncClient;
use sqlx::SqlitePool;

use crate::ai::provider::AiProvider;
use crate::images::ImageStore;

pub struct AiRateLimiter {
    limit: u32,
    counter: AtomicU32,
    window_start: AtomicU64,
}

impl AiRateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            counter: AtomicU32::new(0),
            window_start: AtomicU64::new(0),
        }
    }

    pub fn check(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 60;

        let window = self.window_start.load(Ordering::Relaxed);

        if now != window
            && self
                .window_start
                .compare_exchange(window, now, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            self.counter.store(1, Ordering::Relaxed);
            return true;
        }

        let count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        count <= self.limit
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub image_store: ImageStore,
    pub mqtt_client: Option<AsyncClient>,
    pub mqtt_prefix: String,
    pub mqtt_connected: Option<Arc<AtomicBool>>,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_disabled: bool,
    pub ai_provider: Option<Arc<dyn AiProvider>>,
    pub ai_base_url: String,
    pub ai_model: String,
    pub ai_rate_limiter: Option<Arc<AiRateLimiter>>,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_requests_within_limit() {
        let limiter = AiRateLimiter::new(3);
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(limiter.check());
    }

    #[test]
    fn rejects_requests_over_limit() {
        let limiter = AiRateLimiter::new(2);
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(!limiter.check());
    }

    #[test]
    fn resets_on_new_window() {
        let limiter = AiRateLimiter::new(1);
        assert!(limiter.check());
        assert!(!limiter.check());

        // Simulate a new minute window
        limiter.window_start.store(0, Ordering::Relaxed);
        assert!(limiter.check());
    }
}
