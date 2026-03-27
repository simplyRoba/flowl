# TODO (post-1.0)

## PWA Level 2 — Offline & Install

### A: Service worker foundation + offline shell

- [x] Add a service worker via SvelteKit `src/service-worker.ts`
- [x] Precache static assets (JS, CSS, icons) with cache-first strategy
- [x] Offline fallback page when network is unavailable
- [x] Online/offline indicator in the UI shell

### B: Offline data cache (read-only)

- Cache API responses for plant list and plant detail for offline viewing
- Network-first with stale fallback for API calls
- Decide: service worker intercept vs IndexedDB store

### C: Offline write queue + sync

- Queue care events (watering, log entries) created offline
- Sync queued events when connectivity returns
- Conflict resolution and partial-failure handling
- UX for pending/synced state

## AI — Additional Providers

- Ollama provider implementation (behind the same trait)
- Provider selection via env var or Settings UI

## E2E Tests

- Add Playwright or Cypress for full-stack E2E testing
- Cover critical user flows: plant CRUD, care events, watering status, AI chat, import/export
