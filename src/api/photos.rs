#![allow(clippy::missing_errors_doc)]

use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;

use super::error::ApiError;
use super::plants::{PLANT_SELECT, Plant, PlantRow};
use crate::state::AppState;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 MB

pub async fn upload_photo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Json<Plant>, ApiError> {
    // Verify plant exists, get current photo_path
    let current_photo =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    // Extract file field
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::Validation("No file provided".to_string()))?;

    // Validate content type
    let content_type = field.content_type().unwrap_or("").to_string();

    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => {
            return Err(ApiError::Validation(
                "Invalid file type. Allowed: JPEG, PNG, WebP".to_string(),
            ));
        }
    };

    // Read file data
    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Validate size
    if data.len() > MAX_FILE_SIZE {
        return Err(ApiError::Validation(
            "File too large. Maximum size is 5 MB".to_string(),
        ));
    }

    // Generate filename and write to disk
    let filename = format!("{}.{ext}", uuid::Uuid::new_v4());
    let file_path = state.upload_dir.join(&filename);
    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to save file: {e}")))?;

    // Delete old photo if replacing
    if let Some(old_filename) = current_photo {
        let old_path = state.upload_dir.join(&old_filename);
        let _ = tokio::fs::remove_file(&old_path).await;
    }

    // Update photo_path in DB
    sqlx::query("UPDATE plants SET photo_path = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&filename)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Return updated plant
    let query = format!("{PLANT_SELECT} WHERE p.id = ?");
    let row = sqlx::query_as::<_, PlantRow>(&query)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(Plant::from(row)))
}

pub async fn delete_photo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    // Verify plant exists and has a photo
    let photo_path =
        sqlx::query_scalar::<_, Option<String>>("SELECT photo_path FROM plants WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound("Plant not found".to_string()))?;

    let filename =
        photo_path.ok_or_else(|| ApiError::NotFound("Plant has no photo".to_string()))?;

    // Set photo_path = NULL in DB
    sqlx::query("UPDATE plants SET photo_path = NULL, updated_at = datetime('now') WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Delete file from disk
    let file_path = state.upload_dir.join(&filename);
    let _ = tokio::fs::remove_file(&file_path).await;

    Ok(StatusCode::NO_CONTENT)
}
