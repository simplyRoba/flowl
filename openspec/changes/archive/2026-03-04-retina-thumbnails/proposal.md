## Why

Thumbnails appear pixelated on high-DPI devices (e.g. iPhone 15 Pro at 3x). The 200px thumbnail is used for containers displayed at 120 CSS px, which need 360 physical pixels on a 3x screen — 80% more than available. The 600px thumbnail similarly falls short for the plant detail hero (300 CSS px = 900 physical pixels at 3x). The thumbnail system has no awareness of device pixel ratio.

## What Changes

- Add a third thumbnail tier at 1000px alongside the existing 200px and 600px
- Use `srcset` with all three sizes on every `<img>` that loads thumbnails via `thumbUrl`, letting the browser pick the optimal size based on device pixel ratio and container width
- Update `thumbUrl` or add a helper to generate srcset strings
- Update backend thumbnail generation, deletion, orphan cleanup, and migration to handle the new size

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `core/image-store`: Thumbnail generation, deletion, orphan cleanup, and migration must handle three sizes (200, 600, 1000) instead of two
- `ui/image-thumbnails`: Images must use `srcset` with all three thumbnail sizes instead of a single `src`; `thumbUrl` utility needs a companion for srcset generation

## Impact

- **Backend** (`src/images.rs`): `THUMBNAIL_SIZES` constant, `generate_thumbnails`, `thumbnail_paths`, `thumbnail_base_stem`, `is_thumbnail_filename`, deletion, orphan cleanup, and missing-thumbnail migration — all driven by the constant so changes are minimal
- **Frontend** (`ui/src/lib/thumbUrl.ts`): New srcset helper
- **Frontend components**: `+page.svelte` (attention cards, plant grid), `plants/[id]/+page.svelte` (hero photo, timeline photos), `care-journal/+page.svelte` (log entry photos) — all `<img>` tags using `thumbUrl`
- **Storage**: One additional JPEG per photo on disk (marginal)
- **Existing thumbnails**: Startup migration will auto-generate the new 1000px variant for existing photos
