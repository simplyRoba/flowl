use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use axum::extract::FromRef;
use rumqttc::AsyncClient;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub upload_dir: PathBuf,
    pub mqtt_client: Option<AsyncClient>,
    pub mqtt_prefix: String,
    pub mqtt_connected: Option<Arc<AtomicBool>>,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_disabled: bool,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
