use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use axum::extract::FromRef;
use rumqttc::AsyncClient;
use sqlx::SqlitePool;

use crate::ai::provider::AiProvider;
use crate::images::ImageStore;

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
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
