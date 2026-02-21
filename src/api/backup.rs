#![allow(clippy::missing_errors_doc)]

use std::io::Write;

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use super::error::ApiError;
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
    pub occurred_at: String,
    pub created_at: String,
}

pub async fn export_data(State(state): State<AppState>) -> Result<Response, ApiError> {
    let locations = sqlx::query_as::<_, ExportLocation>("SELECT id, name FROM locations")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let plants = sqlx::query_as::<_, ExportPlant>(
        "SELECT id, name, species, icon, photo_path, location_id, watering_interval_days, \
         last_watered, light_needs, difficulty, pet_safety, growth_speed, soil_type, \
         soil_moisture, notes, created_at, updated_at FROM plants",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let care_events = sqlx::query_as::<_, ExportCareEvent>(
        "SELECT id, plant_id, event_type, notes, occurred_at, created_at FROM care_events",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let data = ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        locations,
        plants,
        care_events,
    };

    let json =
        serde_json::to_string_pretty(&data).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        zip.start_file("data.json", options)
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        // Add photo files
        for plant in &data.plants {
            if let Some(ref photo_path) = plant.photo_path {
                let file_path = state.upload_dir.join(photo_path);
                if let Ok(photo_data) = tokio::fs::read(&file_path).await {
                    let archive_path = format!("photos/{photo_path}");
                    zip.start_file(&archive_path, options)
                        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                    zip.write_all(&photo_data)
                        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
                }
            }
        }

        zip.finish()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }

    Ok((
        [
            (header::CONTENT_TYPE, "application/zip"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"flowl-export.zip\"",
            ),
        ],
        buf,
    )
        .into_response())
}
