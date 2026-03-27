## Context

flowl is a SvelteKit SPA (SSR disabled, `adapter-static`) embedded in a Rust binary via `rust-embed`. The app is already installable as a standalone PWA with a manifest, icons, theme-color handling, and pull-to-refresh. However, there is no service worker — every page load fetches all assets over the network and offline access shows a raw browser error.

The shell layout (`ui/src/routes/+layout.svelte`) already tracks standalone PWA state and touch capability via `ui/src/lib/pull-to-refresh.ts`. The layout is client-rendered with data fetching driven by `onMount` and store-based API calls. No online/offline detection exists anywhere in the UI.

SvelteKit provides built-in service worker support: creating `ui/src/service-worker.ts` makes the framework compile it with access to `$service-worker` modules that expose the build manifest for precaching.

## Goals / Non-Goals

**Goals:**

- Register a service worker that precaches the static build output (JS, CSS, HTML shell, icons, manifest).
- Serve cached assets with a cache-first strategy so installed-PWA users get instant loads on repeat visits.
- Serve a friendly offline fallback page when the network is unreachable and no cached response exists for a navigation request.
- Show an online/offline connectivity indicator in the app shell.
- Detect service worker updates and prompt the user to refresh when a new app version is available.

**Non-Goals:**

- No caching of API responses (`/api/*`) or uploaded images (`/uploads/*`) — that is the scope of a future offline data cache change.
- No offline write queuing or background sync.
- No push notifications.
- No custom cache strategies beyond cache-first for static assets.

## Decisions

### 1. Use SvelteKit's built-in service worker support

**Decision:** Create `ui/src/service-worker.ts` using the `$service-worker` module imports rather than a manual Workbox or vite-plugin-pwa setup.

**Why:** SvelteKit's built-in support automatically provides the build manifest (`files`, `build`, `routes`) at compile time, so precaching stays in sync with the actual build output without maintaining a separate manifest. It keeps the dependency footprint at zero — no Workbox runtime, no additional Vite plugins.

**Alternatives considered:**

- vite-plugin-pwa / Workbox: more features out of the box but adds a runtime dependency, an extra build step, and configuration surface that isn't needed for precaching static assets.
- Manual service worker in `ui/static/sw.js`: would work but loses build manifest integration, requiring manual cache-busting and version tracking.

### 2. Cache-first for static assets, network-only for everything else

**Decision:** Intercept `fetch` events in the service worker. For requests matching precached assets, serve from cache first (falling back to network). For all other requests (API calls, uploads, external), pass through to network without caching.

**Why:** The static build output is immutable per deployment — filenames include content hashes. Cache-first is safe and gives instant loads. API responses are intentionally excluded from this change to keep scope narrow and avoid stale-data complexity.

**Alternatives considered:**

- Stale-while-revalidate for static assets: unnecessary because the filenames are content-hashed, so a cached file is always correct for that hash.
- Network-first for API with fallback: deferred to the offline data cache change (Change B in TODO.md).

### 3. Offline fallback as a precached HTML page

**Decision:** Include an offline fallback page in the precache set. When a navigation request fails and no cached response exists, serve this page instead of letting the browser show its default error.

**Why:** This is the simplest approach that gives a branded, friendly experience. The fallback page can be a static HTML file in `ui/static/offline.html` added to the precache manifest, or a SvelteKit-rendered page captured at build time. A static file is preferred because it avoids any runtime dependency on the SvelteKit app shell loading successfully.

**Alternatives considered:**

- SvelteKit error page: would require the app shell JS to load, which defeats the purpose when offline and nothing is cached.
- No fallback: rejected because raw browser error pages break the installed-app experience.

### 4. Offline-only dot badge on the Settings nav item

**Decision:** Track connectivity state in the app shell using `navigator.onLine` for the initial value and `online`/`offline` window events for changes. When offline, display a small colored dot badge on the Settings nav item in the bottom navigation bar. When connectivity returns, the dot disappears. There is no "online" indicator — the absence of the dot is the normal state.

**Why:** A dot badge is the least disruptive visual signal: it requires no layout shift, no text, and no dismissal. Placing it on Settings is appropriate because connectivity is a system-level concern, not a per-page concern. The browser APIs are not perfectly reliable (they detect network interface state, not actual server reachability), but they are sufficient for a "you appear to be offline" ambient hint.

**Alternatives considered:**

- Banner across the top of the page: rejected because it disrupts layout and feels heavy-handed for an ambient signal.
- Toast notification on connectivity change: considered but rejected as the sole indicator because it vanishes and gives no persistent signal if the user misses it.
- Periodic health-check pings to `/health`: more accurate but adds network traffic and complexity for minimal gain in this foundation change.
- Service worker message-based connectivity: adds coupling between SW and app; simpler to keep it in the shell with standard browser APIs.

### 5. Service worker update detection via `controllerchange` event

**Decision:** After registering the service worker, listen for the `controllerchange` event on `navigator.serviceWorker`. When a new service worker takes control, show a toast or banner prompting the user to reload. The new service worker should call `skipWaiting()` on activation so the update takes effect immediately rather than waiting for all tabs to close.

**Why:** Without update handling, users of the installed PWA could run stale cached assets indefinitely. The `skipWaiting()` + `controllerchange` pattern is the standard approach and keeps the UX simple: the user sees "Update available" and can tap to reload.

**Alternatives considered:**

- Silent auto-reload: rejected because it could interrupt the user mid-interaction.
- Wait for all tabs to close: impractical for a standalone PWA where there is typically one persistent "tab."
- `updatefound` + `statechange` on the registration: more granular but more complex; `controllerchange` is sufficient for a "reload now" prompt.

### 6. Cache versioning via service worker file hash

**Decision:** The service worker file itself changes whenever the build output changes (because the precache manifest embedded in it changes). The browser treats any byte change in the service worker file as an update, triggering the install/activate lifecycle. Old caches are cleaned up in the `activate` event by deleting any cache names that don't match the current version.

**Why:** This piggybacks on the browser's built-in service worker update mechanism. No manual version strings or external version files are needed.

**Alternatives considered:**

- Manual version constant: error-prone and easy to forget.
- Workbox precache manifest with revision hashes: unnecessary overhead given SvelteKit already provides content-hashed filenames.

## Risks / Trade-offs

- **[Risk] Stale assets after deployment if service worker update is missed** → Mitigation: `skipWaiting()` ensures the new worker activates immediately; old caches are purged in the `activate` handler; the update-available prompt nudges users to reload.
- **[Risk] `navigator.onLine` is unreliable on some platforms (reports online when behind a captive portal)** → Mitigation: the indicator is a hint, not a gate. The app still attempts network requests regardless of the indicator state. Future changes can add server-reachability checks if needed.
- **[Risk] Service worker intercepts requests during development** → Mitigation: SvelteKit only registers the service worker in production builds. The dev server does not serve a service worker. This is built-in behavior.
- **[Risk] iOS Safari has historically had service worker quirks (cache eviction, scope limits)** → Mitigation: keep the precache set small (build assets only), avoid relying on persistent SW storage for critical data, and test on iOS Safari specifically.
- **[Trade-off] `skipWaiting()` means the new service worker activates while the old page is still loaded** → This is acceptable because the static assets are content-hashed: the old page's references still resolve to cached files, and the user reloads at their convenience via the prompt.

## Migration Plan

- No backend changes. The service worker is a purely client-side addition.
- Ship behind no feature flag — service workers are progressive enhancement by nature. Browsers that don't support them simply ignore the registration.
- Rollback: remove `ui/src/service-worker.ts` and the registration code in the layout. The browser will unregister the service worker on the next visit when it finds no service worker file at the registered scope.

## Open Questions

- None.
