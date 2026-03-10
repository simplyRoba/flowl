use std::io::Read;
use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::{Multipart, State};
use serde::{Deserialize, Serialize};

use tracing::info;

use super::care_events::validate_event_type;
use super::error::{ApiError, db_error};
use super::plants::{
    validate_all_care_info, validate_light_needs, validate_required_name,
    validate_watering_interval,
};
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
        return Err(ApiError::BadRequest("IMPORT_VERSION_MISMATCH"));
    }

    if server_parts[0] != archive_parts[0] || server_parts[1] != archive_parts[1] {
        return Err(ApiError::BadRequest("IMPORT_VERSION_MISMATCH"));
    }

    Ok(())
}

fn validate_filename(name: &str) -> Result<(), ApiError> {
    if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
        return Err(ApiError::BadRequest("IMPORT_INVALID_FILENAME"));
    }
    Ok(())
}

fn validate_dest_path(
    dest: &std::path::Path,
    upload_dir: &std::path::Path,
) -> Result<(), ApiError> {
    let canonical_dir = upload_dir.canonicalize().map_err(|e| {
        tracing::error!("Failed to resolve upload dir: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;
    let canonical_dest = dest.canonicalize().or_else(|_| {
        // File doesn't exist yet -- canonicalize the parent and append the filename
        let parent = dest.parent().unwrap_or(dest);
        let name = dest
            .file_name()
            .ok_or(ApiError::BadRequest("IMPORT_INVALID_FILENAME"))?;
        Ok::<_, ApiError>(
            parent
                .canonicalize()
                .map_err(|e| {
                    tracing::error!("Failed to resolve path: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?
                .join(name),
        )
    })?;
    if !canonical_dest.starts_with(&canonical_dir) {
        return Err(ApiError::BadRequest("IMPORT_INVALID_FILENAME"));
    }
    Ok(())
}

type PhotoEntry = (String, Vec<u8>);

/// Parse and validate the ZIP archive synchronously, returning the data and extracted photos.
/// This keeps all `ZipArchive` usage in a non-async context so the future remains Send.
fn parse_archive(bytes: &[u8]) -> Result<(ImportData, Vec<PhotoEntry>), ApiError> {
    const MAX_JSON_SIZE: u64 = 50 * 1024 * 1024; // 50 MB
    const MAX_PHOTO_SIZE: usize = 5 * 1024 * 1024; // 5 MB -- matches upload limit

    let cursor = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|_| ApiError::BadRequest("IMPORT_INVALID_ARCHIVE"))?;

    // Validate all filenames
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_ARCHIVE"))?;
        validate_filename(file.name())?;
    }

    // Read and parse data.json
    let data: ImportData = {
        let data_file = archive
            .by_name("data.json")
            .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_DATA"))?;

        let mut json_bytes = Vec::new();
        data_file
            .take(MAX_JSON_SIZE + 1)
            .read_to_end(&mut json_bytes)
            .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_DATA"))?;
        if json_bytes.len() as u64 > MAX_JSON_SIZE {
            return Err(ApiError::BadRequest("IMPORT_FILE_TOO_LARGE"));
        }
        let json_str = String::from_utf8(json_bytes)
            .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_DATA"))?;

        serde_json::from_str(&json_str).map_err(|_| ApiError::BadRequest("IMPORT_INVALID_DATA"))?
    };

    check_version(&data.version)?;

    // Extract photo files into memory
    let mut photos = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_ARCHIVE"))?;

        let name = file.name().to_string();
        if let Some(filename) = name.strip_prefix("photos/") {
            if filename.is_empty() {
                continue;
            }
            let mut contents = Vec::new();
            file.take(MAX_PHOTO_SIZE as u64 + 1)
                .read_to_end(&mut contents)
                .map_err(|_| ApiError::BadRequest("IMPORT_INVALID_DATA"))?;
            if contents.len() > MAX_PHOTO_SIZE {
                return Err(ApiError::BadRequest("IMPORT_FILE_TOO_LARGE"));
            }
            photos.push((filename.to_string(), contents));
        }
    }

    Ok((data, photos))
}

async fn replace_database(pool: &sqlx::SqlitePool, data: &ImportData) -> Result<(), ApiError> {
    let mut tx = pool.begin().await.map_err(db_error)?;

    // Delete in correct FK order
    sqlx::query("DELETE FROM care_events")
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    sqlx::query("DELETE FROM plants")
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    sqlx::query("DELETE FROM locations")
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

    for loc in &data.locations {
        validate_required_name(&loc.name, "LOCATION_NAME_REQUIRED")?;

        sqlx::query("INSERT INTO locations (id, name) VALUES (?, ?)")
            .bind(loc.id)
            .bind(&loc.name)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
    }

    for plant in &data.plants {
        validate_required_name(&plant.name, "PLANT_NAME_REQUIRED")?;
        validate_watering_interval(plant.watering_interval_days)?;
        validate_light_needs(&plant.light_needs)?;
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
        .map_err(db_error)?;
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
        .map_err(db_error)?;
    }

    tx.commit().await.map_err(db_error)?;

    Ok(())
}

/// # Errors
/// Returns `ApiError::BadRequest` for malformed uploads, invalid archives,
/// or version mismatches, or `ApiError::InternalError` on database failures.
pub async fn import_data(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, ApiError> {
    // Extract file from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?
        .ok_or(ApiError::BadRequest("IMPORT_NO_FILE"))?;

    let bytes = field
        .bytes()
        .await
        .map_err(|_| ApiError::BadRequest("INVALID_REQUEST_BODY"))?;

    info!(size_bytes = bytes.len(), "Data import started");

    // Phase 1: Parse and validate the archive (synchronous -- keeps future Send)
    let (data, photos) = parse_archive(&bytes)?;

    info!(
        version = %data.version,
        locations = data.locations.len(),
        plants = data.plants.len(),
        care_events = data.care_events.len(),
        photos = photos.len(),
        "Archive parsed, replacing database"
    );

    // Phase 2: Write new photos to disk (may overwrite same-named files)
    // Clear thumbnails so they are regenerated in Phase 4b (e.g. after orientation fixes).
    // Original photos are preserved so a failed DB transaction doesn't lose data.
    state.image_store.clear_thumbnails().await;
    let photos_count = photos.len();
    let upload_dir = state.image_store.upload_dir();
    for (filename, contents) in &photos {
        let dest = upload_dir.join(filename);
        validate_dest_path(&dest, upload_dir)?;
        tokio::fs::write(&dest, contents).await.map_err(|e| {
            tracing::error!("Failed to write {filename}: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;
    }

    // Phase 3: Replace database data in a transaction
    replace_database(&state.pool, &data).await?;

    // Phase 4: Clean up photos no longer referenced by the new database
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
                    "watering_interval_days": 7, "light_needs": "indirect",
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

    #[test]
    fn validate_dest_path_allows_file_inside_upload_dir() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("photo.jpg");
        std::fs::write(&dest, b"test").unwrap();
        assert!(super::validate_dest_path(&dest, dir.path()).is_ok());
    }

    #[test]
    fn validate_dest_path_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("..").join("escaped.jpg");
        // Create the file so canonicalize can resolve it
        std::fs::write(&dest, b"test").unwrap();
        assert!(super::validate_dest_path(&dest, dir.path()).is_err());
        // Clean up the escaped file
        let _ = std::fs::remove_file(&dest);
    }

    #[test]
    fn validate_dest_path_works_for_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("new_photo.jpg");
        // File doesn't exist yet -- should still validate via parent
        assert!(super::validate_dest_path(&dest, dir.path()).is_ok());
    }
}
