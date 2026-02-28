## Context

Plant photos are managed in `src/api/photos.rs` with inline file handling: content-type validation, size check, UUID naming, disk write/delete. Phase 9a adds photo support to care events, which needs the same file operations. Rather than duplicate, we extract a shared `ImageStore` and wire both features through it.

Current state: `AppState` holds `upload_dir: PathBuf` and `photos.rs` does all file I/O directly.

## Goals / Non-Goals

**Goals:**
- Shared `ImageStore` that plant photos and care event photos both use
- `photo_path` column on `care_events` with upload/delete endpoints
- `photo_url` in care event API responses
- Photo cleanup on event deletion (direct and cascade)
- Backup/restore includes care event photos

**Non-Goals:**
- Frontend UI for care event photos (Phase 9b)
- Image processing (resizing, thumbnails, format conversion)
- Changing the upload directory structure (all files remain flat in `/data/uploads/`)
- Modifying the AI identify endpoint's photo handling (it doesn't store files)

## Decisions

### 1. `ImageStore` struct in `src/images.rs`

```rust
pub struct ImageStore {
    upload_dir: PathBuf,
}

impl ImageStore {
    pub fn new(upload_dir: PathBuf) -> Self;
    pub fn upload_dir(&self) -> &Path;
    pub async fn save(&self, data: &[u8], content_type: &str) -> Result<String, ImageError>;
    pub async fn delete(&self, filename: &str);
}
```

`save` validates content-type (jpeg/png/webp), validates size (≤ 5 MB), generates a UUID filename, writes to disk, and returns the filename. `delete` removes the file and logs a warning on failure (never errors — deletion is best-effort, matching current behavior).

**Why a struct over free functions:** Encapsulates the `upload_dir` path so callers don't need to know about it. `AppState` holds `image_store: ImageStore` instead of `upload_dir: PathBuf`.

**Why not a trait:** There's only one storage backend (local filesystem) and no plans for S3/etc. A concrete struct is simpler. If cloud storage is added later, it can be made into a trait then.

### 2. Replace `upload_dir` in `AppState` with `ImageStore`

`AppState.upload_dir: PathBuf` becomes `AppState.image_store: ImageStore`. All callers that access `state.upload_dir` switch to `state.image_store`. The `ServeDir` mount in `server.rs` uses `state.image_store.upload_dir()`.

`ImageStore` implements `Clone` (wrapping `upload_dir: PathBuf` which is already `Clone`).

### 3. Care event photo routes mirror plant photo routes

```
POST   /api/plants/:id/care/:event_id/photo    → upload
DELETE /api/plants/:id/care/:event_id/photo    → delete
```

Same multipart handling, same validation rules. The upload handler updates `care_events.photo_path`; the delete handler nulls it. Both verify the event belongs to the plant.

Handlers live in a new section within `src/api/care.rs` (alongside existing care event handlers) rather than a separate file — keeps related code together.

### 4. Photo cleanup on care event deletion

The existing `DELETE /api/plants/:id/care/:event_id` handler needs to check for `photo_path` before deleting the row and call `image_store.delete()` if present. This mirrors how plant deletion cleans up plant photos.

### 5. Startup orphan cleanup

On application boot (after migrations, before HTTP server starts), `ImageStore` scans the uploads directory and deletes any file not referenced by `plants.photo_path` or `care_events.photo_path`.

```rust
impl ImageStore {
    pub async fn cleanup_orphans(&self, pool: &SqlitePool);
}
```

This makes photo cleanup self-healing: if the process crashes between a DB delete and a file delete (e.g. during CASCADE), orphaned files are cleaned up on the next restart. It also eliminates the need to pre-query care event photos before plant deletion — the plant delete handler only needs to clean up the plant's own photo; any care event photo orphans left by CASCADE are handled by the next startup.

**Why startup instead of a background job:** The app has no background tasks today (only MQTT, which is event-driven). A periodic job adds complexity for a problem that only manifests on crashes. Startup cleanup is simpler, guaranteed to run, and sufficient for a single-user app.

**Why query the DB instead of tracking references in the ImageStore:** The DB is the source of truth for which photos are in use. Scanning the uploads directory and cross-referencing against DB columns is straightforward and doesn't require any additional bookkeeping.

### 6. Backup/restore changes

**Export:** After bundling plant photos, also iterate `care_events` rows with non-null `photo_path` and include those files under the same `photos/` directory in the ZIP. No namespace conflict — all filenames are UUIDs.

**Restore:** No structural change needed. The restore process already extracts all files from `photos/` to the upload directory and inserts `care_events` rows with their `photo_path` values. It will naturally handle care event photos once the column exists. Only change: the `data.json` care events entries now may include `photo_path`.

### 7. `ImageError` type

A small enum for `ImageStore` errors:

```rust
pub enum ImageError {
    InvalidContentType,
    TooLarge,
    IoError(std::io::Error),
}
```

Handlers convert `ImageError` to `ApiError` at the call site. This keeps `ImageStore` independent of the HTTP layer.

## Risks / Trade-offs

- **Cascade photo orphans** — CASCADE deletes care_events rows without application-level file cleanup. → Mitigated by startup orphan cleanup, which removes any unreferenced files on the next boot.
- **Flat upload directory** — All photos (plant + care event) share one directory. With many photos, this could slow filesystem listing. → Acceptable for a self-hosted app; unlikely to reach problematic scale.
- **Migration on existing data** — Adding a nullable `photo_path` column to `care_events` is a non-breaking ALTER TABLE. SQLite handles this without rewriting the table. → No risk.
