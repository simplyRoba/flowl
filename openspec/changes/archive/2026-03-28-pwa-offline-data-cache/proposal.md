## Why

The app has a service worker that precaches static assets and shows an offline fallback page, but API data is never cached. When a user loses connectivity — common when tending plants away from Wi-Fi — the dashboard and plant detail pages fail to load entirely, showing only error text. Caching API responses for read-only offline browsing lets users check their plant list, watering status, and care history without a network connection.

## What Changes

- Extend the service worker to intercept GET requests for plant-related API endpoints and thumbnails with a network-first strategy that falls back to cached responses when offline.
- Thumbnail images (`/uploads/*_<size>.jpg`) are cached opportunistically so plant photos remain visible offline.
- Full-size images (`/uploads/<hash>.<ext>` without a size suffix) are not cached — the lightbox requires connectivity.
- Pages that are not cached (care journal, settings) show their existing skeleton/loading UI, then display an offline-appropriate message instead of a generic load error.
- Mutation actions (water, log care event, edit, delete) are disabled while offline with a user-facing indication. Offline write queuing is explicitly out of scope.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ui/pwa`: Add requirements for API response caching (network-first with stale fallback), thumbnail caching, and scope boundaries (which endpoints are cached, which are not).
- `ui/plant-dashboard`: Add requirement that the dashboard is browsable offline using cached data, and that mutation controls (water button) are disabled when offline.
- `ui/plant-detail`: Add requirement that plant detail and its care event list are viewable offline using cached data, and that mutation controls are disabled when offline.
- `ui/care-journal`: Add requirement that the care journal shows an offline message instead of a generic error when the network is unavailable.
- `ui/settings`: Add requirement that settings shows an offline message instead of silently hiding sections when the network is unavailable.

## Impact

- `ui/src/service-worker.ts`: Add fetch intercept rules for API GET endpoints (`/api/plants`, `/api/plants/:id`, `/api/plants/:id/care`, `/api/stats`, `/api/locations`) and thumbnail URLs. Implement network-first caching with stale fallback.
- `ui/src/routes/+page.svelte` (dashboard): Disable water button when offline.
- `ui/src/routes/plants/[id]/+page.svelte` (plant detail): Disable mutation controls when offline.
- `ui/src/routes/care-journal/+page.svelte`: Show offline message when fetch fails and `navigator.onLine` is false.
- `ui/src/routes/settings/+page.svelte`: Show offline message when fetch fails and `navigator.onLine` is false.
- No backend changes. No new runtime dependencies. No IndexedDB. No offline write queue.
