## Context

Photos are stored as UUID-named files (e.g., `a1b2c3.jpg`) in the uploads directory. The `photo_path` column in `plants` and `care_events` stores just the filename. The API derives `photo_url` as `/uploads/{photo_path}` in the `From<PlantRow>` conversion. The frontend uses `photo_url` everywhere — lightbox, hero, grid cards, 72px timeline thumbnails — loading the full-resolution original regardless of display size.

## Goals / Non-Goals

**Goals:**
- Generate 200px and 600px thumbnails alongside originals on every photo upload
- Serve thumbnails via the existing `/uploads/` static file route (no new endpoints)
- Derive thumbnail URLs from `photo_path` by convention (no new DB columns)
- Generate missing thumbnails for existing images on startup
- Keep backup/restore working correctly with thumbnails

**Non-Goals:**
- Dynamic resizing or on-demand generation
- Serving different formats (e.g., converting PNG→WebP)
- Client-side responsive image selection (`srcset`) — we pick the right size in code
- Cropping or aspect ratio changes — thumbnails preserve aspect ratio, fit within the size bound

## Decisions

### 1. Filename convention: `<uuid>_<size>.<ext>`

Thumbnails are derived from the original filename by inserting a size suffix before the extension:
- Original: `a1b2c3.jpg`
- Thumbnails: `a1b2c3_200.jpg`, `a1b2c3_600.jpg`

**Why:** No DB schema changes. Thumbnail URLs are computed from `photo_path` in application code, the same way `photo_url` already is. Deletion just needs to delete `{stem}_{size}.{ext}` alongside the original. Orphan cleanup can identify thumbnails by the `_200`/`_600` suffix and skip them (they're not "orphans" — they're derived from the original).

**Alternative considered:** Separate `thumbs/200/` and `thumbs/600/` subdirectories. Rejected because `ServeDir` already serves from the flat uploads directory — no subdirectory routing needed. Flat is simpler.

### 2. Thumbnail generation in `ImageStore::save()`

After writing the original, decode it with the `image` crate, resize to 200px and 600px (longest edge, preserving aspect ratio), and write both thumbnails as JPEG (quality 80). Thumbnail generation runs on `spawn_blocking` to avoid blocking the Tokio runtime.

**Why JPEG output for all thumbnails:** Thumbnails are for display, not archival. JPEG at quality 80 produces the smallest files with acceptable quality. A 200px PNG thumbnail of a photo would be ~50 KB vs ~15 KB as JPEG. The original is preserved in its uploaded format.

**Why `spawn_blocking`:** The `image` crate's decode/resize is CPU-bound. Running it on the async executor would starve other tasks during the ~20-50ms processing time.

**Failure handling:** If thumbnail generation fails (e.g., corrupt image data), log a warning and continue. The original is already saved successfully. The thumbnail files won't exist on disk, so the frontend's derived URLs will 404 — but this only happens for images the `image` crate can't decode, which are also likely to render poorly as originals.

### 3. No API changes — frontend derives thumbnail URLs

The API continues to return only `photo_url` (the original). The frontend derives thumbnail URLs using the naming convention:

```
photo_url:  /uploads/a1b2c3.jpg
thumb 600:  /uploads/a1b2c3_600.jpg
thumb 200:  /uploads/a1b2c3_200.jpg
```

A shared `thumbUrl(photoUrl, size)` utility in the frontend handles the string manipulation. Each component picks the right size for its display context.

**Why not add `thumb_url`/`thumb_sm_url` to the API?** The backend doesn't know how the frontend displays images. Encoding display sizes into the API response couples the API to UI layout decisions. Adding a new thumbnail size would require API changes. With a convention-based approach, the backend just generates files and the frontend picks what it needs — same pattern CDNs use.

### 4. Orphan cleanup: skip thumbnail files

`cleanup_orphans` currently deletes any file in uploads/ not referenced by `photo_path` in the DB. Thumbnails are not referenced in the DB — they'd be deleted as orphans.

Fix: when checking if a file is referenced, also check if stripping `_200` or `_600` from the stem produces a filename that is referenced. If so, the file is a thumbnail of a referenced original — skip it.

### 5. Startup migration: generate missing thumbnails

On startup, after orphan cleanup, scan all referenced `photo_path` values. For each, check if `_200` and `_600` variants exist on disk. If not, generate them. This handles the upgrade path for existing images.

Runs once per startup but is fast — only processes images that lack thumbnails. After the first run post-upgrade, it's a no-op.

### 6. Backup: exclude thumbnails, restore: regenerate

**Backup:** Only export original files (skip `_200`/`_600` files). Thumbnails are derived data — no need to bloat the archive.

**Restore:** After importing photos, run thumbnail generation for all imported images. This reuses the same startup migration logic.

### 7. `ImageStore::delete()`: delete thumbnails alongside original

When `delete(filename)` is called, also delete `{stem}_200.{ext}` and `{stem}_600.{ext}`. Failures are logged but non-fatal, matching current behavior.

## Risks / Trade-offs

**[Risk] `image` crate increases binary size** → The crate adds ~1-2 MB to the release binary. Acceptable for a Docker image already at ~15 MB. Enable only the JPEG/PNG/WebP decoders to minimize bloat.

**[Risk] Corrupt/unusual images fail to decode** → Some images (e.g., CMYK JPEG, animated WebP) may fail to decode. Mitigation: log a warning, skip thumbnail generation. The frontend uses `photo_url` as fallback — the feature degrades gracefully, never blocks uploads.

**[Risk] Startup migration slow for large collections** → A user with 500 photos would take ~25 seconds on first upgrade (50ms × 500). Mitigation: run on `spawn_blocking`, log progress every 50 images. The server is functional during migration — it just serves originals until thumbnails are ready.

**[Trade-off] JPEG thumbnails for PNG originals lose transparency** → PNG photos with transparency get opaque JPEG thumbnails (white background). Acceptable because plant photos are virtually never transparent. If needed later, PNG thumbnails could be generated for PNG originals.
