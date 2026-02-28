use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use tracing::{info, warn};

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5 MB

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
        tokio::fs::write(self.upload_dir.join(&filename), data)
            .await
            .map_err(ImageError::Io)?;

        Ok(filename)
    }

    /// Delete a file from the uploads directory. Logs a warning on failure.
    pub async fn delete(&self, filename: &str) {
        let path = self.upload_dir.join(filename);
        if let Err(e) = tokio::fs::remove_file(&path).await {
            warn!(filename = %filename, error = %e, "Failed to remove image file");
        }
    }

    /// Remove files in the uploads directory that are not referenced by any
    /// `plants.photo_path` or `care_events.photo_path` row.
    pub async fn cleanup_orphans(&self, pool: &SqlitePool) {
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
            if !referenced.contains(&filename) {
                if let Err(e) = tokio::fs::remove_file(entry.path()).await {
                    warn!(filename = %filename, error = %e, "Failed to remove orphaned image");
                } else {
                    removed += 1;
                }
            }
        }

        if removed > 0 {
            info!(removed, "Cleaned up orphaned image files");
        }
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
    async fn delete_existing_file() {
        let (store, _dir) = temp_store();
        let filename = store.save(b"data", "image/jpeg").await.unwrap();
        assert!(store.upload_dir.join(&filename).exists());
        store.delete(&filename).await;
        assert!(!store.upload_dir.join(&filename).exists());
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
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query("CREATE TABLE plants (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE care_events (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();

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
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query("CREATE TABLE plants (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE care_events (id INTEGER PRIMARY KEY, photo_path TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        let filename = store.save(b"care-photo", "image/webp").await.unwrap();

        sqlx::query("INSERT INTO care_events (id, photo_path) VALUES (1, ?)")
            .bind(&filename)
            .execute(&pool)
            .await
            .unwrap();

        store.cleanup_orphans(&pool).await;

        assert!(store.upload_dir.join(&filename).exists());
    }
}
