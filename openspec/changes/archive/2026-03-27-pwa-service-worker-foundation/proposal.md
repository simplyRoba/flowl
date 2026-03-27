## Why

The app is installable as a standalone PWA but has no service worker. Without one, installed users get a raw browser error when offline and every page load fetches all assets from the network. Adding a service worker with static asset precaching and an offline fallback is the foundational step that enables future offline data viewing and write queuing.

## What Changes

- Add a SvelteKit service worker (`src/service-worker.ts`) that precaches the static build output (JS, CSS, icons, manifest).
- Use a cache-first strategy for static assets so repeat visits load instantly from cache.
- Serve a friendly offline fallback page when the network is unavailable and no cached response exists.
- Add an online/offline status indicator to the UI shell so users know when connectivity is lost.
- Handle service worker updates: when a new app version is detected, notify the user that a refresh is available.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ui/pwa`: Add service worker registration, static asset precaching, offline fallback page, and update notification requirements.
- `ui/shell`: Add online/offline connectivity indicator to the app shell.

## Impact

- New file `ui/src/service-worker.ts` using SvelteKit's built-in service worker support and `$service-worker` module imports.
- Shell layout (`ui/src/routes/+layout.svelte`) gains service worker registration logic and the connectivity indicator.
- New offline fallback page (static HTML or Svelte route) served when network requests fail.
- No backend changes — the Rust server already serves the static build via `rust-embed` and the service worker is a client-side concern.
- No new runtime dependencies expected; SvelteKit provides the service worker build integration.
