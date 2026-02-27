## Why

flowl is a mobile-first plant care app that benefits from home screen installation. Adding a web app manifest makes it installable on iOS and Android via "Add to Home Screen". In standalone mode, the browser chrome disappears entirely — giving users a native-feeling full-screen experience and sidestepping browser-specific rendering quirks (like the Firefox iOS transparent chrome issue).

## What Changes

- Add a `manifest.json` to `ui/static/` with app metadata, icons, theme colors, and `display: standalone`
- Generate PWA icons in required sizes (192x192, 512x512) from the existing favicon SVG
- Add `<link rel="manifest">` to `app.html`
- Add a `TODO.md` documenting Level 2 PWA possibilities (service worker, offline support, caching strategies)

## Capabilities

### New Capabilities

- `ui/pwa`: Web app manifest and installability metadata

### Modified Capabilities

_(none)_

## Impact

- **`ui/static/manifest.json`** — New file with PWA manifest
- **`ui/static/`** — New icon PNG files
- **`ui/src/app.html`** — Add manifest link tag
- **No backend changes**
- **No new dependencies**
