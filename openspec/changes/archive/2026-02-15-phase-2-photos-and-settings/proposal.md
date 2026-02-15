## Why

Phase 2 (Core Plant Management) is ~70% complete with CRUD, list/detail views, and the location backend done. Two features remain: photo upload/storage for plants and a location management section in Settings. Photos give plants a visual identity beyond emoji icons, and the settings page provides a dedicated place to manage locations independently of the plant form.

## What Changes

- Add a `photo_path` column to the `plants` table for storing uploaded photo filenames.
- Add multipart upload and delete endpoints for plant photos (`POST/DELETE /api/plants/:id/photo`).
- Serve uploaded files via `/uploads` route using `tower-http::ServeDir`.
- Introduce `AppState` struct to share the database pool and upload directory path across handlers.
- Add `plant_count` to the location list response via a JOIN query.
- Update the frontend to display photos on dashboard cards and detail views, replacing emoji icons when a photo is present.
- Add a photo section to the plant add/edit form with file upload, preview, and remove functionality.
- Create a Settings page (`/settings`) with location management: list, delete with plant count warnings.

## Capabilities

### New Capabilities

- `ui/settings`: Settings page with location management section.

### Modified Capabilities

- `data/plants`: Add `photo_path` column, upload/delete photo endpoints, photo cleanup on plant deletion.
- `data/locations`: Include `plant_count` in list response.
- `ui/plants`: Photo display on dashboard/detail, photo section in add/edit form.
- `core/server`: `AppState` struct, `/uploads` static file serving.

## Impact

- `Cargo.toml`: Add `axum` multipart feature and `uuid` dependency.
- `migrations/`: New migration for `photo_path` column.
- `src/state.rs`: New module for `AppState`.
- `src/api/photos.rs`: New module for photo upload/delete handlers.
- `src/api/plants.rs`: Add `photo_url` to response, photo cleanup on delete.
- `src/api/locations.rs`: Add `plant_count` to list response.
- `src/server.rs`: Accept `AppState`, mount `/uploads` route.
- `src/main.rs`: Construct `AppState` with upload directory.
- `ui/src/lib/api.ts`: Photo upload/delete functions, updated types.
- `ui/src/lib/stores/plants.ts`: Photo upload/delete store functions.
- `ui/src/lib/components/PlantForm.svelte`: Photo section with upload/preview/remove.
- `ui/src/routes/+page.svelte`: Photo display on dashboard cards.
- `ui/src/routes/plants/[id]/+page.svelte`: Photo display in detail hero.
- `ui/src/routes/settings/+page.svelte`: New settings page with location management.
