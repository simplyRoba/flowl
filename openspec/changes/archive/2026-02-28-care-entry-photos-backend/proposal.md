## Why

Care events have no way to attach photos. Users can't document visual plant state (yellowing leaves, new growth, pest damage) alongside care journal entries. Phase 9a adds the backend foundation: database column, upload/delete endpoints, and photo URLs in API responses. Additionally, the photo handling logic (validation, UUID naming, file write/delete) is currently hardcoded in the plant photo handler — extracting a shared `ImageStore` avoids duplicating that logic for care entry photos (and any future photo-bearing entities).

## What Changes

- Extract shared image storage logic (validate, save, delete) from `src/api/photos.rs` into a new `src/images.rs` module
- Refactor existing plant photo endpoints to use the new `ImageStore`
- Add `photo_path` column to the `care_events` table via migration
- Add `POST /api/plants/:id/care/:event_id/photo` endpoint for uploading a care event photo
- Add `DELETE /api/plants/:id/care/:event_id/photo` endpoint for removing a care event photo
- Include `photo_url` in care event API responses
- Clean up care event photo files on event deletion
- Add startup orphan cleanup — scan uploads directory and delete files not referenced by any `plants.photo_path` or `care_events.photo_path`, making file cleanup self-healing
- Update backup/restore to include care event photos

## Capabilities

### New Capabilities

- `core/image-store`: Shared image storage service — validate content-type/size, save with UUID filename, delete from disk, startup orphan cleanup

### Modified Capabilities

- `data/care-events`: Add photo_path column, photo upload/delete endpoints, photo_url in responses, photo cleanup on deletion
- `core/backup`: Include care event photos in export archive
- `core/restore`: Extract and restore care event photos from import archive

## Impact

- `src/images.rs` — new module: `ImageStore` struct with `save`, `delete`, `validate` methods
- `src/api/photos.rs` — refactored to use `ImageStore` instead of inline file logic
- `src/api/care.rs` — new photo upload/delete handlers, updated response type
- `src/api/backup.rs` — include care event photos in ZIP
- `src/api/restore.rs` — extract care event photos from ZIP
- `src/state.rs` — replace `upload_dir: PathBuf` with `image_store: ImageStore` in `AppState`
- `src/server.rs` — mount care event photo routes, use `image_store.upload_dir()` for ServeDir
- `migrations/` — new migration adding `photo_path` to `care_events`
- No frontend changes (Phase 9b)
- No breaking API changes — `photo_url` is additive (null when absent)
