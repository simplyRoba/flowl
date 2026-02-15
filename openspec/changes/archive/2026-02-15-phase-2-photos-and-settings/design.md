## Context

Phase 2 established plant CRUD, the dashboard, detail view, add/edit forms, and the location backend. Plants currently use Noto emoji icons as their visual representation. This change adds photo upload support and a settings page for location management, completing Phase 2.

## Goals / Non-Goals

**Goals:**
- Photo upload (JPEG/PNG/WebP, max 5 MB) stored on disk, served via `/uploads`
- Photo replaces emoji icon as primary visual when present; emoji remains as fallback
- `AppState` struct to share pool + upload directory across handlers
- `plant_count` in location list response for UI display
- Settings page with location list, plant counts, and delete with confirmation

**Non-Goals:**
- No image resizing or thumbnail generation
- No group-by-location view (deferred)
- No location editing from settings (only delete)
- No multiple photos per plant

## Decisions

### Decision 1: AppState with FromRef

Introduce `AppState { pool, upload_dir }` with `FromRef<AppState> for SqlitePool` so existing handlers extracting `State<SqlitePool>` continue to work without changes. Only handlers that need the upload directory extract `State<AppState>`.

### Decision 2: UUID filenames for uploads

Generate `{uuid}.{ext}` filenames for uploaded photos to avoid collisions and path traversal issues. Store only the filename in `photo_path`; the full URL is computed as `/uploads/{photo_path}` in the response mapping.

### Decision 3: Photo as optional replacement for icon

When `photo_url` is present in the plant response, the frontend shows the photo instead of the emoji icon. The icon field remains in the database and can be restored if the photo is removed. The icon picker is hidden when a photo is set.

### Decision 4: Multipart upload with validation

Use Axum's multipart extractor for photo uploads. Validate content type (JPEG/PNG/WebP) and size (max 5 MB) before writing to disk. Return 422 for invalid uploads.

### Decision 5: plant_count via LEFT JOIN

Add `plant_count` to the location list query using `LEFT JOIN plants` and `COUNT`. This avoids a separate query per location and keeps the response self-contained for the settings UI.

## Risks / Trade-offs

- **No image processing**: Large photos are stored and served as-is. Acceptable for a personal app; could add resizing later.
- **Disk storage**: Photos are stored on the filesystem alongside the database. The upload directory must be persistent in container deployments.
- **No auth on uploads**: Anyone on the local network can upload photos. Acceptable for the single-user Home Assistant add-on use case.
