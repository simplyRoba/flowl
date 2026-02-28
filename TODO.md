# TODO

## PWA Level 2 — Offline Support & Enhanced Features

### Service Worker
- Add a service worker for asset caching (SvelteKit has built-in `src/service-worker.ts` support)
- Cache strategy: cache-first for static assets (JS, CSS, images), network-first for API calls
- Offline fallback page when network is unavailable

### Offline Data
- Cache plant list and detail data for offline viewing
- Queue care events (watering, log entries) created offline and sync when back online
- Show offline indicator in the UI

### Push Notifications
- Watering reminders via push notifications
- Requires backend support for web push subscriptions (VAPID keys, push endpoint storage)

## AI — Additional Providers

- Ollama provider implementation (behind the same trait)
- Provider selection via env var or Settings UI

## PWA Level 2 — Enhanced Install Experience
- Add `beforeinstallprompt` handler for a custom in-app install banner (Android/Chrome only)
- Add Apple-specific meta tags (`apple-mobile-web-app-capable`, `apple-touch-icon`) for better iOS integration
- Add maskable icon variant for Android adaptive icons
