## Context

The backend generates two JPEG thumbnails per uploaded photo (200px and 600px, longest edge). The frontend picks one via `thumbUrl(photoUrl, size)` and sets it as the `src`. On high-DPI devices (2x, 3x), the browser stretches these beyond their native resolution, causing visible pixelation — most noticeably on the attention card (120 CSS px container, 200px thumb, 360 physical pixels needed at 3x).

The `THUMBNAIL_SIZES` constant in `src/images.rs` drives generation, deletion, orphan cleanup, and startup migration — all iterate over it, so adding a size is a one-line backend change.

## Goals / Non-Goals

**Goals:**
- Sharp thumbnails on all device pixel ratios up to 3x
- Let the browser choose the optimal image via `srcset` — no over-fetching on 1x screens
- Minimal implementation surface: one new thumbnail tier, one new frontend helper

**Non-Goals:**
- Art direction / `<picture>` element — all views show the same crop, just at different resolutions
- Lazy loading or intersection observer — separate concern
- Changing JPEG quality or compression strategy

## Decisions

### 1. Add 1000px as the third thumbnail tier

**Choice:** `THUMBNAIL_SIZES: [u32; 3] = [200, 600, 1000]`

**Rationale:** The three tiers map to clear use cases:
- **200px** — small thumbnails (timeline 72px, journal 80px) — sufficient up to 3x
- **600px** — medium views (attention card 120px, form previews) — sufficient up to 3x
- **1000px** — large views (grid cards ~240–400px, detail hero 260–300px) — sufficient up to 3x

A 1000px JPEG from a typical phone photo (3000–4000px) is still a significant reduction in file size. No new dependencies needed — same `image` crate `thumbnail()` call.

**Alternatives considered:**
- Two sizes only (just bump 200→400): Doesn't help the 600px shortfall on large views at 3x
- Four sizes (add both 400 and 1000): Diminishing returns — 600 already covers what 400 would, extra storage per photo for little benefit

### 2. `srcset` with `sizes` attribute on all thumbnail `<img>` tags

**Choice:** Add a `thumbSrcset(photoUrl)` helper that returns `"{base}_200.jpg 200w, {base}_600.jpg 600w, {base}_1000.jpg 1000w"`. Each `<img>` sets `srcset` and a `sizes` attribute matching its CSS container width.

**Rationale:** `srcset` with `w` descriptors lets the browser factor in both container size and device pixel ratio to pick the smallest sufficient image. No JavaScript needed — pure HTML.

**Alternatives considered:**
- `srcset` with `x` descriptors (1x/2x/3x): Doesn't account for container size, so a 72px timeline thumb would load the same image as a 300px hero on a 3x device
- Always load the 600px or 1000px thumb: Wastes bandwidth on 1x devices and small containers

### 3. Keep `thumbUrl` unchanged, add `thumbSrcset` alongside

**Choice:** `thumbUrl` stays as-is for the `src` fallback. New `thumbSrcset(photoUrl): string` generates the srcset string.

**Rationale:** `src` is still needed as a fallback for browsers without srcset support (effectively none today, but it's required HTML). Using `thumbUrl(photo, 200)` as the `src` keeps the smallest image as default. The `onerror` fallback to `photo_url` continues to work unchanged.

## Risks / Trade-offs

- **Storage increase** — One additional JPEG per photo on disk. For a typical phone photo, the 1000px thumb is ~100–200 KB. Acceptable for a self-hosted app with local storage. → No mitigation needed.
- **Startup migration time** — Existing photos will get a new 1000px thumbnail generated on first startup after upgrade. The migration already handles this via `generate_missing_thumbnails`. → Logs progress every 50 photos; runs on `spawn_blocking`.
- **Backup/restore size** — Backup archives will be slightly larger with the extra thumbnails. → Thumbnails are regenerated on restore anyway (restore calls `generate_missing_thumbnails`), so they could be excluded from backups in the future if needed. Not worth optimizing now.
