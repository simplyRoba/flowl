## 1. Extract ImageStore

- [x] 1.1 Create `src/images.rs` with `ImageStore` struct and `ImageError` enum
- [x] 1.2 Implement `ImageStore::save()` — validate content-type, validate size, generate UUID filename, write to disk
- [x] 1.3 Implement `ImageStore::delete()` — remove file, log warning on failure
- [x] 1.4 Add unit tests for `ImageStore` (valid save, invalid content-type, too large, delete)

## 2. Refactor plant photos to use ImageStore

- [x] 2.1 Replace `upload_dir: PathBuf` with `image_store: ImageStore` in `AppState` and update `main.rs`
- [x] 2.2 Update `src/server.rs` to use `state.image_store.upload_dir()` for `ServeDir`
- [x] 2.3 Refactor `src/api/photos.rs` `upload_photo` to call `image_store.save()` and `image_store.delete()`
- [x] 2.4 Refactor `src/api/photos.rs` `delete_photo` to call `image_store.delete()`
- [x] 2.5 Update plant deletion in `src/api/plants.rs` to use `image_store.delete()`
- [x] 2.6 Update `src/api/backup.rs` and `src/api/restore.rs` to use `state.image_store.upload_dir()`

## 3. Care event photo schema and response

- [x] 3.1 Create migration adding `photo_path TEXT` column to `care_events`
- [x] 3.2 Add `photo_path` to `CareEventRow` and `photo_url` to the care event API response struct
- [x] 3.3 Add `photo_url` mapping (from `photo_path`) in care event response construction

## 4. Care event photo endpoints

- [x] 4.1 Add `POST /api/plants/:id/care/:event_id/photo` upload handler in `src/api/care_events.rs`
- [x] 4.2 Add `DELETE /api/plants/:id/care/:event_id/photo` delete handler in `src/api/care_events.rs`
- [x] 4.3 Register care event photo routes in `src/api/mod.rs` with 10 MB `DefaultBodyLimit`

## 5. Photo cleanup

- [x] 5.1 Update care event deletion handler to delete photo file (if any) before removing the row
- [x] 5.2 Implement `ImageStore::cleanup_orphans(&self, pool: &SqlitePool)` — scan uploads dir, query referenced filenames from `plants` and `care_events`, delete unreferenced files
- [x] 5.3 Call `cleanup_orphans` in `main.rs` after migrations, before starting the HTTP server
- [x] 5.4 Add unit test for `cleanup_orphans` (orphaned file deleted, referenced file preserved)

## 6. Backup/restore

- [x] 6.1 Update export to include care event photos in the ZIP `photos/` directory
- [x] 6.2 Add `photo_path` to the care events section of `data.json` export/import serialization

## 7. Verify

- [x] 7.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
