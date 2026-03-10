use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;

use tracing::info;

use super::error::{ApiError, db_error};
use super::plants::{PLANT_SELECT, Plant, PlantRow};
use crate::images::ImageError;
use crate::state::AppState;

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist,
/// `ApiError::Validation` for invalid file types or oversized files, or
/// `ApiError::InternalError` on multipart parsing or database failures.
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
            .map_err(db_error)?
            .ok_or(ApiError::NotFound("PLANT_NOT_FOUND"))?;

    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?
        .ok_or(ApiError::Validation("PHOTO_NO_FILE"))?;

    let content_type = field.content_type().unwrap_or("").to_string();
    let data = field
        .bytes()
        .await
        .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?;

    let filename = state
        .image_store
        .save(&data, &content_type)
        .await
        .map_err(|e| match e {
            ImageError::InvalidContentType => ApiError::Validation("PHOTO_INVALID_TYPE"),
            ImageError::TooLarge => ApiError::Validation("PHOTO_TOO_LARGE"),
            ImageError::Io(ref io_err) => {
                tracing::error!("Photo save failed: {io_err}");
                ApiError::InternalError("PHOTO_SAVE_FAILED")
            }
        })?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    sqlx::query("UPDATE plants SET photo_path = ?, updated_at = ? WHERE id = ?")
        .bind(&filename)
        .bind(&now)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;

    if let Some(ref old_filename) = current_photo {
        state.image_store.delete(old_filename).await;
    }

    info!(plant_id = id, filename = %filename, "Photo uploaded");

    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;

    Ok(Json(Plant::from(row)))
}

/// # Errors
/// Returns `ApiError::NotFound` if the plant does not exist or has no photo, or
/// `ApiError::InternalError` on database failures.
pub async fn delete_photo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let photo_path =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?
            .ok_or(ApiError::NotFound("PLANT_NOT_FOUND"))?;

    let filename = photo_path.ok_or(ApiError::NotFound("PHOTO_NOT_FOUND"))?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    sqlx::query("UPDATE plants SET photo_path = NULL, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;

    state.image_store.delete(&filename).await;

    info!(plant_id = id, "Photo deleted");
    Ok(StatusCode::NO_CONTENT)
}
