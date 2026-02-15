use std::path::PathBuf;

use axum::extract::FromRef;
use rumqttc::AsyncClient;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub upload_dir: PathBuf,
    pub mqtt_client: Option<AsyncClient>,
    pub mqtt_prefix: String,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
