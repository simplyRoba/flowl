# TODO (post-1.0)

## PWA Level 2 — Offline & Install

### Service Worker & Offline Data

- Add a service worker for asset caching (SvelteKit has built-in `src/service-worker.ts` support)
- Cache strategy: cache-first for static assets (JS, CSS, images), network-first for API calls
- Offline fallback page when network is unavailable
- Cache plant list and detail data for offline viewing
- Queue care events (watering, log entries) created offline and sync when back online
- Show offline indicator in the UI

## AI — Additional Providers

- Ollama provider implementation (behind the same trait)
- Provider selection via env var or Settings UI

## E2E Tests

- Add Playwright or Cypress for full-stack E2E testing
- Cover critical user flows: plant CRUD, care events, watering status, AI chat, import/export
