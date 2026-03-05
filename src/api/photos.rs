use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;

use tracing::info;

use super::error::ApiError;
use super::plants::{PLANT_SELECT, Plant, PlantRow};
use crate::images::ImageError;
use crate::state::AppState;

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist,
/// `ApiError::Validation` for invalid file types or oversized files, or
/// `ApiError::BadRequest` on multipart parsing or database failures.
pub async fn upload_photo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Json<Plant>, ApiError> {
    let current_photo =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("No file provided".to_string()))?;

    let content_type = field.content_type().unwrap_or("").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let filename = state
        .image_store
        .save(&data, &content_type)
        .await
        .map_err(|e| match e {
            ImageError::InvalidContentType | ImageError::TooLarge => {
                ApiError::Validation(e.to_string())
            }
            ImageError::Io(_) => ApiError::BadRequest(format!("Failed to save file: {e}")),
        })?;

    sqlx::query("UPDATE plants SET photo_path = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&filename)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if let Some(ref old_filename) = current_photo {
        state.image_store.delete(old_filename).await;
    }

    info!(plant_id = id, filename = %filename, "Photo uploaded");

    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(Plant::from(row)))
}

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist or has no photo, or
/// `ApiError::BadRequest` on database failures.
pub async fn delete_photo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let photo_path =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let filename =
        photo_path.ok_or_else(|| ApiError::NotFound("Plant has no photo".to_string()))?;

    sqlx::query("UPDATE plants SET photo_path = NULL, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    state.image_store.delete(&filename).await;

    info!(plant_id = id, "Photo deleted");
    Ok(StatusCode::NO_CONTENT)
}
