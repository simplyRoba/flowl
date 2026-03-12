use std::io::Write;

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use tracing::info;

use super::error::{ApiError, db_error};
use crate::images::is_thumbnail_filename;
use crate::state::AppState;

#[derive(Serialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub locations: Vec<ExportLocation>,
    pub plants: Vec<ExportPlant>,
    pub care_events: Vec<ExportCareEvent>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ExportLocation {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ExportPlant {
    pub id: i64,
    pub name: String,
    pub species: Option<String>,
    pub icon: String,
    pub photo_path: Option<String>,
    pub location_id: Option<i64>,
    pub watering_interval_days: i64,
    pub last_watered: Option<String>,
    pub light_needs: String,
    pub difficulty: Option<String>,
    pub pet_safety: Option<String>,
    pub growth_speed: Option<String>,
    pub soil_type: Option<String>,
    pub soil_moisture: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ExportCareEvent {
    pub id: i64,
    pub plant_id: i64,
    pub event_type: String,
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub occurred_at: String,
    pub created_at: String,
}

/// # Errors
/// Returns `ApiError::InternalError` on database failures or if the ZIP archive cannot be created.
pub async fn export_data(State(state): State<AppState>) -> Result<Response, ApiError> {
    let locations = sqlx::query_as::<_, ExportLocation>("SELECT id, name FROM locations")
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?;

    let plants = sqlx::query_as::<_, ExportPlant>(
        "SELECT p.id, p.name, p.species, p.icon, p.photo_path, p.location_id, p.watering_interval_days, \
         lw.last_watered, \
         p.light_needs, p.difficulty, p.pet_safety, p.growth_speed, p.soil_type, \
         p.soil_moisture, p.notes, p.created_at, p.updated_at \
         FROM plants p LEFT JOIN plant_last_watered lw ON lw.plant_id = p.id",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_error)?;

    let care_events = sqlx::query_as::<_, ExportCareEvent>(
        "SELECT id, plant_id, event_type, notes, photo_path, occurred_at, created_at FROM care_events",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(db_error)?;

    let data = ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        locations,
        plants,
        care_events,
    };

    info!(
        locations = data.locations.len(),
        plants = data.plants.len(),
        care_events = data.care_events.len(),
        "Data export started"
    );

    let json = serde_json::to_string_pretty(&data).map_err(|e| {
        tracing::error!("JSON serialization failed: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("data.json", options).map_err(|e| {
            tracing::error!("ZIP write failed: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;
        zip.write_all(json.as_bytes()).map_err(|e| {
            tracing::error!("ZIP write failed: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;

        // Add original photo files (plants + care events), excluding thumbnails
        let photo_paths: Vec<&str> = data
            .plants
            .iter()
            .filter_map(|p| p.photo_path.as_deref())
            .chain(
                data.care_events
                    .iter()
                    .filter_map(|e| e.photo_path.as_deref()),
            )
            .filter(|p| !is_thumbnail_filename(p))
            .collect();

        for photo_path in photo_paths {
            let file_path = state.image_store.upload_dir().join(photo_path);
            if let Ok(photo_data) = tokio::fs::read(&file_path).await {
                let archive_path = format!("photos/{photo_path}");
                zip.start_file(&archive_path, options).map_err(|e| {
                    tracing::error!("ZIP write failed: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?;
                zip.write_all(&photo_data).map_err(|e| {
                    tracing::error!("ZIP write failed: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?;
            }
        }

        zip.finish().map_err(|e| {
            tracing::error!("ZIP finalize failed: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;
    }

    let disposition = format!(
        "attachment; filename=\"flowl-export-v{}.zip\"",
        env!("CARGO_PKG_VERSION")
    );

    Ok((
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        buf,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use crate::images::is_thumbnail_filename;

    #[test]
    fn export_photo_filter_excludes_thumbnails() {
        let all_paths = [
            "abc.jpg",
            "abc_200.jpg",
            "abc_600.jpg",
            "def.png",
            "def_200.jpg",
            "def_600.jpg",
            "ghi.webp",
        ];

        let exported: Vec<&&str> = all_paths
            .iter()
            .filter(|p| !is_thumbnail_filename(p))
            .collect();

        assert_eq!(exported, [&"abc.jpg", &"def.png", &"ghi.webp"]);
    }

    #[test]
    fn export_photo_filter_keeps_all_when_no_thumbnails() {
        let paths = ["photo1.jpg", "photo2.png"];
        let exported: Vec<&&str> = paths.iter().filter(|p| !is_thumbnail_filename(p)).collect();
        assert_eq!(exported, [&"photo1.jpg", &"photo2.png"]);
    }
}
