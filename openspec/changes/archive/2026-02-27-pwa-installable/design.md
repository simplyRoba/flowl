## Context

flowl already has a favicon SVG at `ui/static/favicon.svg` and sets `theme-color` dynamically via JavaScript. Adding PWA installability requires a manifest file, icon PNGs, and a link tag.

## Goals / Non-Goals

**Goals:**

- Make the app installable via "Add to Home Screen" on iOS and Android
- Launch in standalone mode (no browser chrome)
- Provide properly sized icons for home screen and splash screens

**Non-Goals:**

- Offline support / service worker (documented as future TODO)
- Push notifications
- App store distribution
- Apple-specific meta tags beyond basics (`apple-mobile-web-app-capable` etc. are nice-to-have but not required for installability)

## Decisions

### 1. Static manifest in `ui/static/`

**Decision:** Place `manifest.json` as a static file in `ui/static/`. SvelteKit serves static files directly.

**Why:** The manifest is static content — no server-side rendering needed. SvelteKit's static directory is the standard place for this.

### 2. SVG-to-PNG icon generation

**Decision:** Generate 192x192 and 512x512 PNG icons from `favicon.svg` and place them in `ui/static/`. Commit the generated PNGs.

**Why:** PWA manifests require raster icons — SVG is not universally supported in manifest `icons`. Two sizes (192, 512) satisfy the minimum requirements for Chrome and Safari installability prompts.

### 3. Light theme for manifest colors

**Decision:** Use the light theme background (`#FAF6F1`) for `theme_color` and `background_color` in the manifest. The dynamic `theme-color` meta tag (already implemented) handles runtime theme switching.

**Why:** The manifest is static and can't switch between themes. Light is the default theme. The meta tag takes precedence at runtime, so dark mode users still get the correct chrome color.

## Risks / Trade-offs

**[Risk] iOS limitations** → iOS Safari has limited PWA support (no install prompt, manual "Add to Home Screen" only). This is an Apple platform limitation, not something we can fix. The manifest still works — users just need to know how to add it manually.
