## Context

The service worker (`ui/src/service-worker.ts`) currently handles three request categories: static assets (cache-first), navigation (cache-first with offline fallback), and everything else (pass-through). API calls and uploaded images fall into the pass-through bucket — when the network is unavailable, fetches fail, stores surface error messages, and the user sees a broken page.

The app shell already tracks online/offline state via `navigator.onLine` and window events in `+layout.svelte`, exposing an `isOffline` reactive variable. This signal is available for disabling controls but is not currently threaded to child routes or components.

All API access goes through `ui/src/lib/api.ts` which uses a single `request()` helper wrapping the browser `fetch` API. Stores call into `api.ts`, catch errors, and surface messages via `resolveError()`. There is no retry logic, no caching layer, and no distinction between "offline" and "server error" in the error handling path.

Thumbnails are served at `/uploads/<hash>_<size>.jpg` (sizes 200, 600, 1000). Full-size originals are at `/uploads/<hash>.<ext>`. Every `<img>` using thumbnails has an `onerror` fallback to the original URL.

## Goals / Non-Goals

**Goals:**

- Cache API responses for dashboard and plant detail so both pages are browsable offline.
- Cache thumbnail images so plant photos remain visible offline.
- Apply a network-first strategy: always prefer fresh data, serve stale only when the network is unavailable.
- Disable mutation controls (water, care event forms) when offline.
- Show an offline-specific message on pages that are intentionally not cached (care journal, settings).

**Non-Goals:**

- No offline write queue or background sync — mutations require connectivity.
- No IndexedDB or app-layer caching — all caching happens in the service worker via the Cache API.
- No cache expiration or TTL logic — stale data is acceptable since the user is offline and the indicator dot communicates the state.
- No caching of full-size original images — the lightbox requires connectivity.
- No caching of AI, MQTT, or settings endpoints — these are either non-essential offline or are infrastructure-status data that is meaningless without connectivity.

## Decisions

### 1. Service worker intercept using the Cache API (no IndexedDB)

**Decision:** Extend the existing service worker fetch handler to intercept specific API GET requests and thumbnail URLs. Use the Cache API to store successful responses. No app-layer caching, no IndexedDB, no changes to stores or `api.ts`.

**Why:** The caching is read-only and the data model is simple — full HTTP responses keyed by URL. The Cache API is purpose-built for this. The stores and `api.ts` don't need to know about caching at all, keeping the change small and the existing code untouched. This was chosen over IndexedDB because there is no offline write queue (Phase C was dropped), so there's no need for structured data storage or local-state mutation.

**Alternatives considered:**

- IndexedDB in stores: each store function would double in size with cache-read/write logic. Adds ~150-200 lines across 4 stores, introduces a new dependency (`idb` wrapper), and creates a second source of truth. Only justified if offline writes were planned.
- Hybrid (SW cache + app-layer awareness via custom headers): considered adding an `X-Flowl-Source: cache` response header so stores could detect stale data. Rejected because the existing offline dot indicator is sufficient — adding per-response staleness signals would require store changes for no user-visible benefit.

### 2. Separate cache for API responses and thumbnails

**Decision:** Use a dedicated cache named `flowl-api-${version}` for API responses and thumbnails, separate from the existing `flowl-cache-${version}` used for static assets. Clean up old API caches in the activate handler alongside the existing static cache cleanup.

**Why:** API responses and thumbnails have different lifecycles than static assets. Static assets are immutable (content-hashed filenames) and precached at install time. API responses are dynamic and cached opportunistically during use. Separating them avoids the API cache growing the precache set and makes it possible to reason about each cache independently. The version suffix ensures stale API caches are cleaned up on service worker updates, matching the existing cleanup pattern.

**Alternatives considered:**

- Single shared cache: simpler but mixes immutable precached assets with dynamic API responses, making cache inspection and debugging harder.
- Unversioned API cache (persistent across SW updates): would preserve offline data across app updates, but risks serving API responses shaped for an older API contract. Safer to clear on update and re-populate on next online visit.

### 3. Network-first with stale fallback for API endpoints

**Decision:** For API GET requests matching the cached endpoint list, try the network first. On success, clone the response into the API cache and return it. On failure (network error or timeout), fall back to the cached response if one exists.

Cached endpoints:
- `/api/plants` — plant list (dashboard)
- `/api/plants/<id>` — single plant detail
- `/api/plants/<id>/care` — care events for a plant
- `/api/stats` — dashboard statistics
- `/api/locations` — location list (used in dashboard grouping)

The endpoint matching uses URL pathname pattern matching: `/api/plants` exactly, `/api/plants/<number>` for detail, `/api/plants/<number>/care` for care events.

**Why:** Network-first ensures the user always sees fresh data when online. The stale fallback only activates when the network is genuinely unavailable, so there is no risk of silently showing stale data while online. This is the standard progressive-enhancement pattern for read-only offline support.

**Alternatives considered:**

- Stale-while-revalidate: returns cached data immediately and refreshes in the background. Faster perceived loads but risks showing stale watering status when the user knows they just watered on another device. Not worth the confusion for a plant care app where status accuracy matters.
- Cache-first with network update: same issue as stale-while-revalidate — shows stale data by default.

### 4. Cache-first for thumbnails, skip originals

**Decision:** Intercept GET requests matching `/uploads/*_<size>.jpg` (thumbnail pattern) with a cache-first strategy. Full-size originals (`/uploads/*` without a size suffix) are not intercepted — they pass through to the network.

**Why:** Thumbnails are small (200px, 600px variants), relatively static (a plant's photo doesn't change frequently), and are shown on every page that needs offline support. Cache-first avoids redundant network fetches for images the user has already seen. Full-size originals can be several MB and are only loaded in the lightbox — caching them would consume significant storage for a feature that's acceptable to skip offline.

The thumbnail URL pattern (`_200.jpg`, `_600.jpg`, `_1000.jpg`) is deterministic from the `thumbUrl()` function, so pattern matching is reliable.

**Alternatives considered:**

- Network-first for thumbnails: unnecessary overhead — thumbnails are effectively immutable for a given photo. If the plant's photo changes, the hash changes, so the URL changes and the old cached thumbnail naturally stops being referenced.
- Cache all uploads including originals: storage concern on mobile devices, especially iOS Safari which aggressively evicts cache storage under pressure. Not worth the risk for lightbox-only content.

### 5. Disable mutations via the existing `isOffline` signal

**Decision:** Thread the `isOffline` reactive state from the layout to child components that have mutation controls. When offline, disable water buttons and care event forms with visual indication. No toast on tap — the disabled state plus the existing offline dot in the nav bar is sufficient.

**Why:** This is the lightest-touch approach. The `isOffline` variable already exists in the layout and is reactive. Components just need to read it and conditionally disable interactive elements. No new state management, no new components.

The specific controls to disable:
- Dashboard: water button on each plant card
- Plant detail: water button, care event creation (the inline care entry form)

**Alternatives considered:**

- Toast notification when tapping a disabled button ("You're offline"): considered but rejected as redundant — the button being visually disabled combined with the offline dot communicates the state. Adding toasts would create noise on every tap.
- Hide mutation controls entirely when offline: hiding controls changes the page layout and could confuse users who expect to see them. Disabled-but-visible is clearer.

### 6. Offline message for uncached pages (care journal, settings)

**Decision:** On the care journal and settings pages, when a data fetch fails and `navigator.onLine` is false, show an offline-specific message instead of the generic error text from `resolveError()`. The message should be i18n-aware (using existing translations infrastructure) and simply state that the page requires connectivity.

**Why:** The current behavior on these pages when offline is either a generic error string (care journal) or silently empty sections (settings). Neither communicates "you're offline" clearly. A dedicated message is a small UX improvement that leverages the translation system already in place.

This does not require service worker changes — it's a UI-only concern in the catch blocks of the page components' data loading functions.

**Alternatives considered:**

- Cache care journal and settings data too: increases the cache scope and service worker complexity for pages that provide limited value offline. The care journal is a paginated, filterable list that's hard to cache meaningfully. Settings is infrastructure status that's irrelevant without connectivity.
- Show the offline.html fallback page: too aggressive — the app shell still works, only the data fetch fails. The user should stay in the app with a contextual message, not be kicked to a separate page.

## Risks / Trade-offs

- **[Risk] iOS Safari aggressively evicts Cache API storage under storage pressure** → Mitigation: the cached data set is small (JSON API responses + JPEG thumbnails for plants the user has viewed). Total cache size is unlikely to exceed a few MB even with dozens of plants. If evicted, the user simply sees fetch errors — same as today.
- **[Risk] Cached API responses may have a different shape after an API change** → Mitigation: the API cache is versioned with `flowl-api-${version}` and cleaned up on service worker activation, same as static assets. A new deployment clears stale API responses.
- **[Risk] `navigator.onLine` can report false positives (online behind captive portal)** → Mitigation: this is an existing limitation from the Phase A design. Mutation controls rely on `navigator.onLine` which is imperfect but acceptable as a hint. The server will reject requests if actually unreachable, and the existing error handling surfaces that.
- **[Trade-off] No stale-data indicator** → The user sees cached data without knowing it's stale. Accepted because the offline dot in the nav bar communicates the connectivity state, and stale plant data (watering status from the last online visit) is still useful for reference.
- **[Trade-off] Clearing API cache on every SW update means the first offline visit after an update shows no cached data** → Accepted because updates are infrequent and the cache repopulates on the next online visit. The alternative (persistent cache) risks stale API shapes.

## Migration Plan

- No backend changes. Purely client-side service worker and component updates.
- No feature flag needed — the caching is progressive enhancement. If the Cache API fails or is unavailable, the app behaves exactly as it does today.
- Rollback: revert the service worker changes. Old API caches are cleaned up automatically on the next service worker activation.

## Open Questions

- None.
