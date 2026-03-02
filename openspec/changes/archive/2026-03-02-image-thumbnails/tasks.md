## 1. Setup

- [x] 1.1 Add `image` crate to `Cargo.toml` with only JPEG, PNG, and WebP decoder/encoder features enabled

## 2. Backend — Thumbnail Generation

- [x] 2.1 Add a `generate_thumbnails(path: &Path)` function in `src/images.rs` that decodes an image file and writes 200px and 600px JPEG thumbnails (quality 80, longest-edge fit, aspect ratio preserved) using the `{stem}_{size}.jpg` naming convention
- [x] 2.2 Call `generate_thumbnails` from `ImageStore::save()` via `spawn_blocking` after writing the original; log warning and continue on failure
- [x] 2.3 Update `ImageStore::delete()` to also delete `{stem}_200.jpg` and `{stem}_600.jpg` alongside the original, silently ignoring missing thumbnails
- [x] 2.4 Write tests for thumbnail generation: JPEG/PNG/WebP originals produce `_200.jpg` and `_600.jpg`, aspect ratio is preserved, corrupt image logs warning without failing save, delete removes thumbnails

## 3. Backend — Orphan Cleanup

- [x] 3.1 Update `cleanup_orphans` to skip files whose stem matches `{referenced_stem}_200` or `{referenced_stem}_600`; delete thumbnail files whose base stem is not referenced
- [x] 3.2 Write tests for orphan cleanup: thumbnails of referenced files are preserved, thumbnails of unreferenced files are deleted

## 4. Backend — Startup Migration

- [x] 4.1 Add a `generate_missing_thumbnails` function that scans all `photo_path` values from `plants` and `care_events`, checks for missing `_200`/`_600` variants on disk, and generates them; log progress every 50 images
- [x] 4.2 Call `generate_missing_thumbnails` on startup after orphan cleanup
- [x] 4.3 Write tests for startup migration: missing thumbnails are generated, existing thumbnails are skipped, missing originals are skipped with warning

## 5. Backend — Backup & Restore

- [x] 5.1 Update backup export to skip files matching `_200.jpg` or `_600.jpg` suffix when collecting photos for the ZIP
- [x] 5.2 Update restore import to call `generate_missing_thumbnails` after extracting photos
- [x] 5.3 Write/update tests: exported ZIP does not contain thumbnail files, restore generates thumbnails for imported photos

## 6. Frontend — Thumbnail Utility

- [x] 6.1 Create `thumbUrl(photoUrl: string, size: number): string` utility in `$lib/utils` that inserts `_{size}` before the extension and changes the extension to `.jpg`
- [x] 6.2 Write tests for `thumbUrl`: derives correct URLs for `.jpg`, `.png`, `.webp` originals

## 7. Frontend — Component Updates

- [x] 7.1 Update plant dashboard grid cards to use `thumbUrl(photo_url, 600)` for the card photo
- [x] 7.2 Update attention cards to use `thumbUrl(photo_url, 200)` for the icon photo
- [x] 7.3 Update plant detail hero to use `thumbUrl(photo_url, 600)`; keep lightbox opening with the original `photo_url`
- [x] 7.4 Update plant detail care timeline thumbnails (72px) to use `thumbUrl(photo_url, 200)`; keep lightbox opening with the original `photo_url`
- [x] 7.5 Update global care journal thumbnails (80px) to use `thumbUrl(photo_url, 200)`; keep lightbox opening with the original `photo_url`
- [x] 7.6 Add `onerror` fallback on all thumbnail `<img>` elements to swap `src` to the original `photo_url` if the thumbnail 404s
- [x] 7.7 Update frontend component tests to verify thumbnail URLs are used and `onerror` fallback is wired

## 8. Quality & Verification

- [x] 8.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` — all must pass
- [x] 8.2 Run `npm run check` in `ui/` — must pass
- [x] 8.3 Run full frontend test suite (`npm test` in `ui/`) — all must pass
