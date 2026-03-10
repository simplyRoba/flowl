use axum::Json;
use axum::extract::State;
use serde::Serialize;
use sqlx::SqlitePool;

use super::error::{ApiError, db_error};

#[derive(Serialize)]
#[allow(clippy::struct_field_names)]
pub struct Stats {
    pub plant_count: i64,
    pub care_event_count: i64,
    pub location_count: i64,
    pub photo_count: i64,
}

/// # Errors
/// Returns `ApiError::InternalError` on database failures.
pub async fn get_stats(State(pool): State<SqlitePool>) -> Result<Json<Stats>, ApiError> {
    let plant_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plants")
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;
    let care_event_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM care_events")
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;
    let location_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM locations")
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;
    let photo_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM plants WHERE photo_path IS NOT NULL")
            .fetch_one(&pool)
            .await
            .map_err(db_error)?;

    Ok(Json(Stats {
        plant_count,
        care_event_count,
        location_count,
        photo_count,
    }))
}
