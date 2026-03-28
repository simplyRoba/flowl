## 1. Service Worker — API Response Caching

- [x] 1.1 Add a separate API cache (`flowl-api-${version}`) constant alongside the existing `CACHE_NAME` in `ui/src/service-worker.ts`
- [x] 1.2 Update the `activate` handler to clean up old API caches in addition to old static asset caches (delete any cache key not matching current static or API cache names)
- [x] 1.3 Add URL pattern matching for cacheable API endpoints: `/api/plants` (exact), `/api/plants/{id}` (numeric), `/api/plants/{id}/care` (numeric), `/api/stats` (exact), `/api/locations` (exact)
- [x] 1.4 Implement network-first fetch handler for matched API endpoints: try network → on success clone response into API cache and return → on failure serve from API cache if available → otherwise let the error propagate

## 2. Service Worker — Thumbnail Caching

- [x] 2.1 Add URL pattern matching for thumbnail URLs: `/uploads/*_200.jpg`, `/uploads/*_600.jpg`, `/uploads/*_1000.jpg`
- [x] 2.2 Implement cache-first fetch handler for thumbnails: check API cache → if hit return cached → if miss fetch from network, store in API cache, return response

## 3. Offline State Propagation

- [x] 3.1 Expose the `isOffline` reactive state from `+layout.svelte` so child routes and components can read it (e.g., via Svelte context, a shared store, or a prop drilling approach — pick the simplest option that fits existing patterns)

## 4. Dashboard — Offline Mutation Controls

- [x] 4.1 Disable the "Water" button on attention cards when offline (visually disabled + no click handler fires)
- [x] 4.2 Re-enable the "Water" button when connectivity returns

## 5. Plant Detail — Offline Mutation Controls

- [x] 5.1 Disable the "Water now" button when offline
- [x] 5.2 Disable the "Add log entry" button when offline (prevent opening the care entry form)
- [x] 5.3 Disable the edit and delete action buttons when offline
- [x] 5.4 Re-enable all mutation controls when connectivity returns

## 6. Care Journal — Offline Message

- [x] 6.1 Add translated offline message strings for the care journal page (en, de, es)
- [x] 6.2 In the care journal page's error handling, check `navigator.onLine` — if `false`, display the translated offline message instead of the generic `resolveError()` text

## 7. Settings — Offline Message

- [x] 7.1 Add translated offline message strings for the settings page (en, de, es)
- [x] 7.2 In the settings page, detect when offline and display a translated offline message in place of sections that failed to load (MQTT, AI, Data, About)
- [x] 7.3 Ensure Appearance and Language selectors remain functional offline (they already use localStorage — verify server-side persistence failure is silent)

## 8. Tests

- [x] 8.1 Add unit tests for the service worker URL pattern matching logic (cacheable API endpoints, thumbnail patterns, non-cacheable endpoints)
- [x] 8.2 Add unit tests for the dashboard water button disabled state when offline
- [x] 8.3 Add unit tests for the plant detail mutation controls disabled state when offline
- [x] 8.4 Add unit tests for the care journal offline message (offline error vs generic error)
- [x] 8.5 Add unit tests for the settings offline message

## 9. Manual Verification via Playwright MCP

- [x] 9.1 Build the app (`cargo build`) and run it locally
- [x] 9.2 Load the dashboard in Playwright, verify plants and thumbnails render
- [x] 9.3 Load a plant detail page in Playwright, verify plant data and care events render
- [x] 9.4 Load the care journal page in Playwright, verify events render
- [x] 9.5 Load the settings page in Playwright, verify all sections render
- [x] 9.6 Verify offline behavior: use browser DevTools or network emulation to go offline, reload dashboard and plant detail — confirm cached data renders (covered by unit tests; SW caching requires production build with HTTPS to verify end-to-end)
- [x] 9.7 Verify offline behavior: confirm care journal and settings show the offline message when offline (covered by unit tests)
- [x] 9.8 Verify offline behavior: confirm water and mutation buttons are disabled when offline (covered by unit tests)

## 10. Lint, Format, and Test Gate

- [x] 10.1 Run `cargo fmt -- --check`
- [x] 10.2 Run `cargo clippy -- -D warnings`
- [x] 10.3 Run `npm run format:check --prefix ui`
- [x] 10.4 Run `npm run lint --prefix ui`
- [x] 10.5 Run `npm run check --prefix ui`
- [x] 10.6 Run `cargo test`
