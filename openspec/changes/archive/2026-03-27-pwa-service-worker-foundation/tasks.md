## 1. Service worker core

- [x] 1.1 Create `ui/src/service-worker.ts` with install handler that precaches static assets from `$service-worker` build/files manifests and the offline fallback page
- [x] 1.2 Add activate handler that deletes old caches not matching the current version and calls `skipWaiting()`
- [x] 1.3 Add fetch handler: cache-first for precached assets, pass-through for everything else (API, uploads, external)
- [x] 1.4 Add fetch handler logic for navigation requests: serve cached response if available, otherwise try network, fall back to offline page on failure

## 2. Offline fallback page

- [x] 2.1 Create `ui/static/offline.html` — a self-contained branded page with inline styles that tells the user they are offline

## 3. Service worker registration and update notification

- [x] 3.1 Add service worker registration in the shell layout on mount (production builds only)
- [x] 3.2 Listen for `controllerchange` on `navigator.serviceWorker` and show an update-available toast when a new worker takes control (skip on first registration when no previous controller exists)

## 4. Offline connectivity indicator

- [x] 4.1 Add `isOffline` reactive state to the shell layout, initialized from `navigator.onLine` and updated via `online`/`offline` window event listeners with cleanup
- [x] 4.2 Add a dot badge to the Settings nav item that is visible only when `isOffline` is true — style it as a small colored dot positioned on the nav icon

## 5. Tests

- [x] 5.1 Add unit tests for the offline dot badge visibility (online vs offline state)
- [x] 5.2 Add unit tests for update notification logic (controller change triggers toast, first registration does not)

## 6. Validation

- [x] 6.1 Run `npm run check --prefix ui`, `npm run lint --prefix ui`, `npm run format:check --prefix ui`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`
