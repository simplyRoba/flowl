use std::io::Read;
use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::{Multipart, State};
use serde::{Deserialize, Serialize};

use tracing::info;

use super::care_events::validate_event_type;
use super::error::ApiError;
use super::plants::{validate_all_care_info, validate_required_name, validate_watering_interval};
use crate::mqtt;
use crate::state::AppState;

#[derive(Deserialize)]
struct ImportData {
    version: String,
    locations: Vec<ImportLocation>,
    plants: Vec<ImportPlant>,
    care_events: Vec<ImportCareEvent>,
}

#[derive(Deserialize)]
struct ImportLocation {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct ImportPlant {
    id: i64,
    name: String,
    species: Option<String>,
    icon: String,
    photo_path: Option<String>,
    location_id: Option<i64>,
    watering_interval_days: i64,
    light_needs: String,
    difficulty: Option<String>,
    pet_safety: Option<String>,
    growth_speed: Option<String>,
    soil_type: Option<String>,
    soil_moisture: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
struct ImportCareEvent {
    id: i64,
    plant_id: i64,
    event_type: String,
    notes: Option<String>,
    #[serde(default)]
    photo_path: Option<String>,
    occurred_at: String,
    created_at: String,
}

#[derive(Serialize)]
pub struct ImportResult {
    pub locations: usize,
    pub plants: usize,
    pub care_events: usize,
    pub photos: usize,
}

fn check_version(archive_version: &str) -> Result<(), ApiError> {
    let server_version = env!("CARGO_PKG_VERSION");
    let server_parts: Vec<&str> = server_version.split('.').collect();
    let archive_parts: Vec<&str> = archive_version.split('.').collect();

    if server_parts.len() < 2 || archive_parts.len() < 2 {
        return Err(ApiError::BadRequest(format!(
            "Invalid version format: expected '{server_version}', got '{archive_version}'"
        )));
    }

    if server_parts[0] != archive_parts[0] || server_parts[1] != archive_parts[1] {
        return Err(ApiError::BadRequest(format!(
            "Version mismatch: server is {server_version}, archive is {archive_version}"
        )));
    }

    Ok(())
}

fn validate_filename(name: &str) -> Result<(), ApiError> {
    if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
        return Err(ApiError::BadRequest(format!(
            "Invalid filename in archive: {name}"
        )));
    }
    Ok(())
}

type PhotoEntry = (String, Vec<u8>);

/// Parse and validate the ZIP archive synchronously, returning the data and extracted photos.
/// This keeps all `ZipArchive` usage in a non-async context so the future remains Send.
fn parse_archive(bytes: &[u8]) -> Result<(ImportData, Vec<PhotoEntry>), ApiError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| ApiError::BadRequest(format!("Invalid ZIP archive: {e}")))?;

    // Validate all filenames
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| ApiError::BadRequest(format!("Invalid archive entry: {e}")))?;
        validate_filename(file.name())?;
    }

    // Read and parse data.json
    let data: ImportData = {
        let mut data_file = archive
            .by_name("data.json")
            .map_err(|_| ApiError::BadRequest("Archive missing data.json".to_string()))?;

        let mut json_str = String::new();
        data_file
            .read_to_string(&mut json_str)
            .map_err(|e| ApiError::BadRequest(format!("Failed to read data.json: {e}")))?;

        serde_json::from_str(&json_str)
            .map_err(|e| ApiError::BadRequest(format!("Invalid data.json: {e}")))?
    };

    check_version(&data.version)?;

    // Extract photo files into memory
    let mut photos = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ApiError::BadRequest(format!("Invalid archive entry: {e}")))?;

        let name = file.name().to_string();
        if let Some(filename) = name.strip_prefix("photos/") {
            if filename.is_empty() {
                continue;
            }
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| ApiError::BadRequest(format!("Failed to read {name}: {e}")))?;
            photos.push((filename.to_string(), contents));
        }
    }

    Ok((data, photos))
}

async fn replace_database(pool: &sqlx::SqlitePool, data: &ImportData) -> Result<(), ApiError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Delete in correct FK order
    sqlx::query("DELETE FROM care_events")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    sqlx::query("DELETE FROM plants")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    sqlx::query("DELETE FROM locations")
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    for loc in &data.locations {
        validate_required_name("Location", &loc.name)?;

        sqlx::query("INSERT INTO locations (id, name) VALUES (?, ?)")
            .bind(loc.id)
            .bind(&loc.name)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }

    for plant in &data.plants {
        validate_required_name("Plant", &plant.name)?;
        validate_watering_interval(plant.watering_interval_days)?;
        validate_all_care_info(
            plant.difficulty.as_deref(),
            plant.pet_safety.as_deref(),
            plant.growth_speed.as_deref(),
            plant.soil_type.as_deref(),
            plant.soil_moisture.as_deref(),
        )?;

        sqlx::query(
            "INSERT INTO plants (id, name, species, icon, photo_path, location_id, \
             watering_interval_days, light_needs, difficulty, pet_safety, \
             growth_speed, soil_type, soil_moisture, notes, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(plant.id)
        .bind(&plant.name)
        .bind(&plant.species)
        .bind(&plant.icon)
        .bind(&plant.photo_path)
        .bind(plant.location_id)
        .bind(plant.watering_interval_days)
        .bind(&plant.light_needs)
        .bind(&plant.difficulty)
        .bind(&plant.pet_safety)
        .bind(&plant.growth_speed)
        .bind(&plant.soil_type)
        .bind(&plant.soil_moisture)
        .bind(&plant.notes)
        .bind(&plant.created_at)
        .bind(&plant.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }

    for event in &data.care_events {
        validate_event_type(&event.event_type)?;

        sqlx::query(
            "INSERT INTO care_events (id, plant_id, event_type, notes, photo_path, occurred_at, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.id)
        .bind(event.plant_id)
        .bind(&event.event_type)
        .bind(&event.notes)
        .bind(&event.photo_path)
        .bind(&event.occurred_at)
        .bind(&event.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(())
}

/// # Errors
/// Returns `ApiError::BadRequest` for malformed uploads, invalid archives,
/// or version mismatches, or on database failures.
pub async fn import_data(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, ApiError> {
    // Extract file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    let bytes = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    info!(size_bytes = bytes.len(), "Data import started");

    // Phase 1: Parse and validate the archive (synchronous — keeps future Send)
    let (data, photos) = parse_archive(&bytes)?;

    info!(
        version = %data.version,
        locations = data.locations.len(),
        plants = data.plants.len(),
        care_events = data.care_events.len(),
        photos = photos.len(),
        "Archive parsed, replacing database"
    );

    // Phase 2: Clear existing uploads and write new photos (before DB commit)
    // Writing first ensures the DB never references files that don't exist.
    state.image_store.clear().await;
    let photos_count = photos.len();
    for (filename, contents) in &photos {
        let dest = state.image_store.upload_dir().join(filename);
        tokio::fs::write(&dest, contents)
            .await
            .map_err(|e| ApiError::BadRequest(format!("Failed to write {filename}: {e}")))?;
    }

    // Phase 3: Replace database data in a transaction
    replace_database(&state.pool, &data).await?;

    // Phase 4: Clean up old photos that are no longer referenced
    state.image_store.cleanup_orphans(&state.pool).await;

    // Phase 4b: Generate thumbnails for imported photos
    state
        .image_store
        .generate_missing_thumbnails(&state.pool)
        .await;

    // Phase 5: Trigger MQTT repair
    if !state.mqtt_disabled {
        let connected = state
            .mqtt_connected
            .as_ref()
            .is_some_and(|b| b.load(Ordering::Relaxed));

        if connected && let Some(client) = state.mqtt_client.as_ref() {
            mqtt::repair(
                &state.pool,
                client,
                &state.mqtt_host,
                state.mqtt_port,
                &state.mqtt_prefix,
            )
            .await;
        }
    }

    info!(
        locations = data.locations.len(),
        plants = data.plants.len(),
        care_events = data.care_events.len(),
        photos = photos_count,
        "Data import complete"
    );

    Ok(Json(ImportResult {
        locations: data.locations.len(),
        plants: data.plants.len(),
        care_events: data.care_events.len(),
        photos: photos_count,
    }))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::images::ImageStore;

    fn tiny_jpeg() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(100, 80, image::Rgb([0, 128, 0]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
        buf.into_inner()
    }

    async fn test_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE plants (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE care_events (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn restore_generates_thumbnails_for_imported_photos() {
        let dir = tempfile::tempdir().unwrap();
        let store = ImageStore::new(dir.path().to_path_buf());
        let pool = test_pool().await;

        // Simulate restored photo on disk
        let photo_data = tiny_jpeg();
        let photo_name = "restored.jpg";
        std::fs::write(dir.path().join(photo_name), &photo_data).unwrap();

        // Simulate DB reference (as restore would insert)
        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, ?)")
            .bind(photo_name)
            .execute(&pool)
            .await
            .unwrap();

        // This is what restore calls after extracting photos
        store.generate_missing_thumbnails(&pool).await;

        assert!(dir.path().join("restored_200.jpg").exists());
        assert!(dir.path().join("restored_600.jpg").exists());
    }

    #[test]
    fn parse_archive_extracts_photos_from_zip() {
        let jpeg_data = tiny_jpeg();
        let mut buf = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            let options = zip::write::SimpleFileOptions::default();

            let data_json = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "locations": [],
                "plants": [{
                    "id": 1, "name": "Fern", "species": null, "icon": "🌿",
                    "photo_path": "test.jpg", "location_id": null,
                    "watering_interval_days": 7, "light_needs": "medium",
                    "difficulty": null, "pet_safety": null, "growth_speed": null,
                    "soil_type": null, "soil_moisture": null, "notes": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }],
                "care_events": []
            });

            zip.start_file("data.json", options).unwrap();
            zip.write_all(data_json.to_string().as_bytes()).unwrap();

            zip.start_file("photos/test.jpg", options).unwrap();
            zip.write_all(&jpeg_data).unwrap();

            zip.finish().unwrap();
        }

        let (data, photos) = match super::parse_archive(&buf) {
            Ok(v) => v,
            Err(_) => panic!("parse_archive failed"),
        };
        assert_eq!(data.plants.len(), 1);
        assert_eq!(photos.len(), 1);
        assert_eq!(photos[0].0, "test.jpg");
        assert_eq!(photos[0].1, jpeg_data);
    }
}
