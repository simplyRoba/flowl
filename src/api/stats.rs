#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use sqlx::SqlitePool;

use super::error::ApiError;

#[derive(Serialize)]
pub struct Stats {
    pub plant_count: i64,
    pub care_event_count: i64,
}

pub async fn get_stats(State(pool): State<SqlitePool>) -> Result<Json<Stats>, ApiError> {
    let plant_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plants")
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let care_event_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM care_events")
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(Stats {
        plant_count,
        care_event_count,
    }))
}
