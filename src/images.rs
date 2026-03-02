use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use tracing::{info, warn};

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 MB
const THUMBNAIL_SIZES: [u32; 2] = [200, 600];
const JPEG_QUALITY: u8 = 80;

#[derive(Debug)]
pub enum ImageError {
    InvalidContentType,
    TooLarge,
    Io(std::io::Error),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidContentType => {
                write!(f, "Invalid file type. Allowed: JPEG, PNG, WebP")
            }
            Self::TooLarge => write!(f, "File too large. Maximum size is 5 MB"),
            Self::Io(e) => write!(f, "File I/O error: {e}"),
        }
    }
}

/// Generate thumbnail variants for an image file on disk.
/// Writes `{stem}_{size}.jpg` for each size in `THUMBNAIL_SIZES`.
fn generate_thumbnails(original_path: &Path) {
    let img = match image::open(original_path) {
        Ok(img) => img,
        Err(e) => {
            warn!(
                path = %original_path.display(),
                error = %e,
                "Failed to decode image for thumbnail generation"
            );
            return;
        }
    };

    let stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let parent = original_path.parent().unwrap_or(Path::new("."));

    for size in THUMBNAIL_SIZES {
        let thumb = img.thumbnail(size, size);
        let thumb_path = parent.join(format!("{stem}_{size}.jpg"));

        let mut buf = std::io::BufWriter::new(match std::fs::File::create(&thumb_path) {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    path = %thumb_path.display(),
                    error = %e,
                    "Failed to create thumbnail file"
                );
                continue;
            }
        });

        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY);
        if let Err(e) = thumb.write_with_encoder(encoder) {
            warn!(
                path = %thumb_path.display(),
                error = %e,
                "Failed to write thumbnail"
            );
            let _ = std::fs::remove_file(&thumb_path);
        }
    }
}

/// Return the thumbnail paths for a given original filename.
fn thumbnail_paths(filename: &str) -> Vec<String> {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    THUMBNAIL_SIZES
        .iter()
        .map(|size| format!("{stem}_{size}.jpg"))
        .collect()
}

/// Check if a filename is a generated thumbnail (has `_200` or `_600` suffix and `.jpg` extension).
pub fn is_thumbnail_filename(filename: &str) -> bool {
    thumbnail_base_stem(filename).is_some()
        && Path::new(filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
}

/// Check if a filename looks like a thumbnail (ends with `_200` or `_600` before extension).
/// If so, return the base stem (without the size suffix).
fn thumbnail_base_stem(filename: &str) -> Option<String> {
    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str())?;
    for size in THUMBNAIL_SIZES {
        let suffix = format!("_{size}");
        if stem.ends_with(&suffix) {
            return Some(stem[..stem.len() - suffix.len()].to_string());
        }
    }
    None
}

#[derive(Clone)]
pub struct ImageStore {
    upload_dir: PathBuf,
}

impl ImageStore {
    pub fn new(upload_dir: PathBuf) -> Self {
        Self { upload_dir }
    }

    pub fn upload_dir(&self) -> &Path {
        &self.upload_dir
    }

    /// Save image bytes to disk after validating content-type and size.
    /// Returns the generated filename (e.g. `<uuid>.jpg`).
    /// Also generates 200px and 600px JPEG thumbnails alongside the original.
    ///
    /// # Errors
    /// Returns `ImageError::InvalidContentType` if the content-type is not
    /// JPEG, PNG, or WebP, `ImageError::TooLarge` if the data exceeds 5 MB,
    /// or `ImageError::Io` on file-write failures.
    pub async fn save(&self, data: &[u8], content_type: &str) -> Result<String, ImageError> {
        let ext = match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => return Err(ImageError::InvalidContentType),
        };

        if data.len() > MAX_FILE_SIZE {
            return Err(ImageError::TooLarge);
        }

        let filename = format!("{}.{ext}", uuid::Uuid::new_v4());
        let original_path = self.upload_dir.join(&filename);
        tokio::fs::write(&original_path, data)
            .await
            .map_err(ImageError::Io)?;

        let path_for_thumbs = original_path.clone();
        if let Err(e) = tokio::task::spawn_blocking(move || {
            generate_thumbnails(&path_for_thumbs);
        })
        .await
        {
            warn!(error = %e, "Thumbnail generation task failed");
        }

        Ok(filename)
    }

    /// Delete a file from the uploads directory, including any thumbnail variants.
    /// Logs a warning on failure.
    pub async fn delete(&self, filename: &str) {
        let path = self.upload_dir.join(filename);
        if let Err(e) = tokio::fs::remove_file(&path).await {
            warn!(filename = %filename, error = %e, "Failed to remove image file");
        }
        for thumb_name in thumbnail_paths(filename) {
            let thumb_path = self.upload_dir.join(&thumb_name);
            if thumb_path.exists()
                && let Err(e) = tokio::fs::remove_file(&thumb_path).await
            {
                warn!(filename = %thumb_name, error = %e, "Failed to remove thumbnail");
            }
        }
    }

    /// Remove files in the uploads directory that are not referenced by any
    /// `plants.photo_path` or `care_events.photo_path` row.
    /// Thumbnail files whose base original is referenced are preserved.
    pub async fn cleanup_orphans(&self, pool: &SqlitePool) {
        let referenced = Self::referenced_photos(pool).await;

        // Build a set of referenced stems for thumbnail matching
        let referenced_stems: HashSet<String> = referenced
            .iter()
            .filter_map(|f| {
                Path::new(f)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
            })
            .collect();

        let Ok(mut entries) = tokio::fs::read_dir(&self.upload_dir).await else {
            return;
        };

        let mut removed = 0u64;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let Ok(file_type) = entry.file_type().await else {
                continue;
            };
            if !file_type.is_file() {
                continue;
            }
            let filename = entry.file_name().to_string_lossy().to_string();

            // Skip if directly referenced
            if referenced.contains(&filename) {
                continue;
            }

            // Skip if this is a thumbnail of a referenced file
            if let Some(base_stem) = thumbnail_base_stem(&filename)
                && referenced_stems.contains(&base_stem)
            {
                continue;
            }

            if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                warn!(filename = %filename, error = %e, "Failed to remove orphaned image");
            } else {
                removed += 1;
            }
        }

        if removed > 0 {
            info!(removed, "Cleaned up orphaned image files");
        }
    }

    /// Generate thumbnails for all referenced photos that are missing them.
    pub async fn generate_missing_thumbnails(&self, pool: &SqlitePool) {
        let referenced = Self::referenced_photos(pool).await;
        if referenced.is_empty() {
            return;
        }

        let mut generated = 0u64;
        let total = referenced.len();

        for (i, filename) in referenced.iter().enumerate() {
            let original_path = self.upload_dir.join(filename);
            if !original_path.exists() {
                warn!(filename = %filename, "Referenced photo not found on disk, skipping thumbnail generation");
                continue;
            }

            let needs_generation = thumbnail_paths(filename)
                .iter()
                .any(|t| !self.upload_dir.join(t).exists());

            if !needs_generation {
                continue;
            }

            let path = original_path.clone();
            if let Err(e) = tokio::task::spawn_blocking(move || {
                generate_thumbnails(&path);
            })
            .await
            {
                warn!(filename = %filename, error = %e, "Thumbnail generation task failed");
            }

            generated += 1;

            if (i + 1) % 50 == 0 {
                info!(progress = i + 1, total, "Thumbnail migration progress");
            }
        }

        if generated > 0 {
            info!(generated, "Generated missing thumbnails");
        }
    }

    async fn referenced_photos(pool: &SqlitePool) -> HashSet<String> {
        let mut referenced: HashSet<String> = HashSet::new();

        if let Ok(rows) = sqlx::query_scalar::<_, String>(
            "SELECT photo_path FROM plants WHERE photo_path IS NOT NULL",
        )
        .fetch_all(pool)
        .await
        {
            referenced.extend(rows);
        }

        if let Ok(rows) = sqlx::query_scalar::<_, String>(
            "SELECT photo_path FROM care_events WHERE photo_path IS NOT NULL",
        )
        .fetch_all(pool)
        .await
        {
            referenced.extend(rows);
        }

        referenced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (ImageStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = ImageStore::new(dir.path().to_path_buf());
        (store, dir)
    }

    /// Create a minimal valid JPEG in memory.
    fn tiny_jpeg() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(100, 80, image::Rgb([0, 128, 0]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
        buf.into_inner()
    }

    /// Create a minimal valid PNG in memory.
    fn tiny_png() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(100, 80, image::Rgb([0, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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
    async fn save_valid_jpeg() {
        let (store, _dir) = temp_store();
        let filename = store.save(b"fake-jpeg", "image/jpeg").await.unwrap();
        assert_eq!(Path::new(&filename).extension().unwrap(), "jpg");
        assert!(store.upload_dir.join(&filename).exists());
    }

    #[tokio::test]
    async fn save_valid_png() {
        let (store, _dir) = temp_store();
        let filename = store.save(b"fake-png", "image/png").await.unwrap();
        assert_eq!(Path::new(&filename).extension().unwrap(), "png");
    }

    #[tokio::test]
    async fn save_valid_webp() {
        let (store, _dir) = temp_store();
        let filename = store.save(b"fake-webp", "image/webp").await.unwrap();
        assert_eq!(Path::new(&filename).extension().unwrap(), "webp");
    }

    #[tokio::test]
    async fn save_rejects_invalid_content_type() {
        let (store, _dir) = temp_store();
        let result = store.save(b"data", "text/plain").await;
        assert!(matches!(result, Err(ImageError::InvalidContentType)));
    }

    #[tokio::test]
    async fn save_rejects_oversized_file() {
        let (store, _dir) = temp_store();
        let data = vec![0u8; MAX_FILE_SIZE + 1];
        let result = store.save(&data, "image/jpeg").await;
        assert!(matches!(result, Err(ImageError::TooLarge)));
        // No file should have been written
        assert_eq!(std::fs::read_dir(store.upload_dir()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn save_generates_thumbnails_for_jpeg() {
        let (store, _dir) = temp_store();
        let data = tiny_jpeg();
        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn save_generates_thumbnails_for_png() {
        let (store, _dir) = temp_store();
        let data = tiny_png();
        let filename = store.save(&data, "image/png").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        // Thumbnails are always JPEG
        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn save_preserves_aspect_ratio_in_thumbnails() {
        let (store, _dir) = temp_store();
        // Create a 3000x2000 image
        let img = image::RgbImage::from_pixel(3000, 2000, image::Rgb([128, 128, 128]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
        let data = buf.into_inner();

        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        let thumb_200 = image::open(store.upload_dir.join(format!("{stem}_200.jpg"))).unwrap();
        assert!(thumb_200.width() <= 200);
        assert!(thumb_200.height() <= 200);
        // Aspect ratio: 3:2 → 200x133
        assert_eq!(thumb_200.width(), 200);
        assert_eq!(thumb_200.height(), 133);

        let thumb_600 = image::open(store.upload_dir.join(format!("{stem}_600.jpg"))).unwrap();
        assert_eq!(thumb_600.width(), 600);
        assert_eq!(thumb_600.height(), 400);
    }

    #[tokio::test]
    async fn save_with_corrupt_image_still_saves_original() {
        let (store, _dir) = temp_store();
        // Not valid image data, but valid content-type
        let filename = store.save(b"not-an-image", "image/jpeg").await.unwrap();
        // Original saved
        assert!(store.upload_dir.join(&filename).exists());
        // No thumbnails (decode failed)
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();
        assert!(!store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(!store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn delete_existing_file() {
        let (store, _dir) = temp_store();
        let filename = store.save(b"data", "image/jpeg").await.unwrap();
        assert!(store.upload_dir.join(&filename).exists());
        store.delete(&filename).await;
        assert!(!store.upload_dir.join(&filename).exists());
    }

    #[tokio::test]
    async fn delete_removes_thumbnails() {
        let (store, _dir) = temp_store();
        let data = tiny_jpeg();
        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(store.upload_dir.join(format!("{stem}_600.jpg")).exists());

        store.delete(&filename).await;

        assert!(!store.upload_dir.join(&filename).exists());
        assert!(!store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(!store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn delete_nonexistent_file_does_not_panic() {
        let (store, _dir) = temp_store();
        store.delete("nonexistent.jpg").await;
        // Should complete without error
    }

    #[tokio::test]
    async fn cleanup_orphans_removes_unreferenced_files() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        // Save two files
        let referenced = store.save(b"keep", "image/jpeg").await.unwrap();
        let orphaned = store.save(b"delete", "image/png").await.unwrap();

        // Reference only one in the DB
        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, ?)")
            .bind(&referenced)
            .execute(&pool)
            .await
            .unwrap();

        assert!(store.upload_dir.join(&referenced).exists());
        assert!(store.upload_dir.join(&orphaned).exists());

        store.cleanup_orphans(&pool).await;

        // Referenced file preserved, orphaned file removed
        assert!(store.upload_dir.join(&referenced).exists());
        assert!(!store.upload_dir.join(&orphaned).exists());
    }

    #[tokio::test]
    async fn cleanup_orphans_preserves_care_event_photos() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        let filename = store.save(b"care-photo", "image/webp").await.unwrap();

        sqlx::query("INSERT INTO care_events (id, photo_path) VALUES (1, ?)")
            .bind(&filename)
            .execute(&pool)
            .await
            .unwrap();

        store.cleanup_orphans(&pool).await;

        assert!(store.upload_dir.join(&filename).exists());
    }

    #[tokio::test]
    async fn cleanup_orphans_preserves_thumbnails_of_referenced_files() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        let data = tiny_jpeg();
        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        // Reference in DB
        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, ?)")
            .bind(&filename)
            .execute(&pool)
            .await
            .unwrap();

        store.cleanup_orphans(&pool).await;

        // Original + thumbnails preserved
        assert!(store.upload_dir.join(&filename).exists());
        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn cleanup_orphans_removes_thumbnails_of_unreferenced_files() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        let data = tiny_jpeg();
        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        // Don't reference in DB — all files are orphans
        store.cleanup_orphans(&pool).await;

        assert!(!store.upload_dir.join(&filename).exists());
        assert!(!store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(!store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn generate_missing_thumbnails_creates_missing() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        // Manually write an image without going through save (simulates pre-existing photo)
        let img = image::RgbImage::from_pixel(400, 300, image::Rgb([255, 0, 0]));
        let filename = format!("{}.jpg", uuid::Uuid::new_v4());
        let path = store.upload_dir.join(&filename);
        img.save(&path).unwrap();

        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        // No thumbnails exist yet
        assert!(!store.upload_dir.join(format!("{stem}_200.jpg")).exists());

        // Reference in DB
        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, ?)")
            .bind(&filename)
            .execute(&pool)
            .await
            .unwrap();

        store.generate_missing_thumbnails(&pool).await;

        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
        assert!(store.upload_dir.join(format!("{stem}_600.jpg")).exists());
    }

    #[tokio::test]
    async fn generate_missing_thumbnails_skips_existing() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        let data = tiny_jpeg();
        let filename = store.save(&data, "image/jpeg").await.unwrap();
        let stem = Path::new(&filename).file_stem().unwrap().to_str().unwrap();

        // Thumbnails already exist from save
        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());

        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, ?)")
            .bind(&filename)
            .execute(&pool)
            .await
            .unwrap();

        // Should be a no-op
        store.generate_missing_thumbnails(&pool).await;

        assert!(store.upload_dir.join(format!("{stem}_200.jpg")).exists());
    }

    #[tokio::test]
    async fn generate_missing_thumbnails_skips_missing_original() {
        let (store, _dir) = temp_store();
        let pool = test_pool().await;

        // Reference a file that doesn't exist on disk
        sqlx::query("INSERT INTO plants (id, photo_path) VALUES (1, 'missing.jpg')")
            .execute(&pool)
            .await
            .unwrap();

        // Should not panic — just logs warning
        store.generate_missing_thumbnails(&pool).await;
    }

    #[test]
    fn is_thumbnail_filename_detects_200() {
        assert!(is_thumbnail_filename("abc_200.jpg"));
    }

    #[test]
    fn is_thumbnail_filename_detects_600() {
        assert!(is_thumbnail_filename("abc_600.jpg"));
    }

    #[test]
    fn is_thumbnail_filename_rejects_original() {
        assert!(!is_thumbnail_filename("abc.jpg"));
    }

    #[test]
    fn is_thumbnail_filename_rejects_non_jpg_thumbnail() {
        assert!(!is_thumbnail_filename("abc_200.png"));
    }

    #[test]
    fn is_thumbnail_filename_rejects_other_suffix() {
        assert!(!is_thumbnail_filename("abc_400.jpg"));
    }
}
