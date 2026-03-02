## Why

The UI loads full-resolution photos (up to 5 MB each) for every display context — including 64×64 px timeline thumbnails, ~240 px plant grid cards, and 36 px care form previews. This wastes bandwidth, slows page loads, and hurts perceived performance, especially on mobile connections. Generating fixed-size thumbnails on upload eliminates this waste.

## What Changes

- Extend `ImageStore` to generate thumbnail variants (200px and 600px) on every photo save, stored alongside the original with a size suffix (e.g., `<uuid>_200.jpg`)
- Serve thumbnails via the existing `/uploads/` static file route — no new endpoints needed
- Add a startup migration that generates thumbnails for all existing photos that lack them
- Update all frontend components to use thumbnail URLs (derived from `photo_url` by convention) for their display context
- Update backup/restore to include thumbnails in export and regenerate them on import
- Update orphan cleanup to account for thumbnail files

## Capabilities

### New Capabilities

- `ui/image-thumbnails`: Shared `thumbUrl` utility for deriving thumbnail URLs from `photo_url`, and `onerror` fallback to original when a thumbnail fails to load

### Modified Capabilities

- `core/image-store`: Add thumbnail generation on save, thumbnail-aware orphan cleanup, and startup thumbnail migration for existing images
- `core/backup`: Include thumbnail files in the exported ZIP alongside originals
- `core/restore`: Regenerate thumbnails after import (or extract them from the archive)
- `ui/plant-dashboard`: Use 600px thumbnails for plant grid cards, 200px for attention card icons
- `ui/plant-detail`: Use 600px for hero photo, 200px for care timeline entries
- `ui/care-journal`: Use 200px thumbnails for event entries
- `ui/care-entry-form`: Use 200px thumbnail for photo preview

## Impact

- **New dependency**: `image` crate (Rust) for decoding/resizing JPEG, PNG, and WebP
- **Backend**: `src/images.rs` (thumbnail generation, cleanup), `src/api/backup.rs`, `src/api/restore.rs`
- **Frontend**: `ui/src/routes/+page.svelte`, `ui/src/routes/plants/[id]/+page.svelte`, `ui/src/routes/care-journal/+page.svelte`, `ui/src/lib/components/CareEntryForm.svelte`
- **Disk**: ~70 KB additional per photo (200px ~15 KB + 600px ~55 KB), negligible vs 1-5 MB originals
- **Build**: `image` crate adds ~2–3s compile time
- **Migration**: Startup job generates thumbnails for existing photos on first run after upgrade
