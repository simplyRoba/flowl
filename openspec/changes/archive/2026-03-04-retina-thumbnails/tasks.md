## 1. Backend — add 1000px thumbnail tier

- [x] 1.1 Change `THUMBNAIL_SIZES` in `src/images.rs` from `[200, 600]` to `[200, 600, 1000]`
- [x] 1.2 Update the doc comment on `ImageStore::save()` to mention three sizes
- [x] 1.3 Update existing tests that assert on thumbnail filenames to include `_1000.jpg` (`save_generates_thumbnails_for_jpeg`, `save_generates_thumbnails_for_png`, `save_preserves_aspect_ratio_in_thumbnails`, `save_applies_exif_orientation_to_thumbnails`, `save_with_corrupt_image_still_saves_original`, `delete_removes_thumbnails`, `cleanup_orphans_preserves_thumbnails_of_referenced_files`, `cleanup_orphans_removes_thumbnails_of_unreferenced_files`, `generate_missing_thumbnails_creates_missing`, `generate_missing_thumbnails_skips_existing`, `is_thumbnail_filename_*`)

## 2. Frontend — srcset utility

- [x] 2.1 Add `thumbSrcset(photoUrl: string): string` to `ui/src/lib/thumbUrl.ts` returning `"{base}_200.jpg 200w, {base}_600.jpg 600w, {base}_1000.jpg 1000w"`
- [x] 2.2 Add unit tests for `thumbSrcset` (JPEG and PNG inputs)

## 3. Frontend — update img tags to use srcset

- [x] 3.1 Attention cards in `ui/src/routes/+page.svelte`: add `srcset` via `thumbSrcset`, set `sizes="120px"`, keep `src={thumbUrl(photo, 200)}`
- [x] 3.2 Plant grid cards in `ui/src/routes/+page.svelte`: add `srcset`, set `sizes` to match grid column width
- [x] 3.3 Plant detail hero in `ui/src/routes/plants/[id]/+page.svelte`: add `srcset`, set `sizes` with breakpoints matching the CSS (260px default, 300px at 1280px+, 100vw at 768px-)
- [x] 3.4 Timeline care photos in `ui/src/routes/plants/[id]/+page.svelte`: add `srcset`, set `sizes="72px"`
- [x] 3.5 Care journal photos in `ui/src/routes/care-journal/+page.svelte`: add `srcset`, set `sizes="80px"`

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
- [x] 4.2 Run `ui/npm run check`
