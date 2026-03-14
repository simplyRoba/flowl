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

## Accessibility

- Run Lighthouse accessibility audit on each page and fix flagged issues

## AI — Identify Improvements

- Allow AI to report "not a plant" — the strict JSON schema forces the AI to always return plant suggestions, even for non-plant photos. Add an optional error/message field so the AI can decline identification.

## Care Journal — Event Grouping

- Group repeated watering events by plant into summaries (e.g. "Watered 3 times in the last 3 weeks") instead of showing each one individually
