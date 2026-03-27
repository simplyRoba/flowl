## ADDED Requirements

### Requirement: Service worker registration

The app SHALL register a service worker on page load in production builds.

#### Scenario: Service worker registered in production

- **WHEN** the app loads in a production build
- **THEN** the app SHALL register a service worker via `navigator.serviceWorker.register`

#### Scenario: No service worker in development

- **WHEN** the app loads in a development build
- **THEN** no service worker SHALL be registered

### Requirement: Static asset precaching

The service worker SHALL precache all static build assets so they are available without network access on subsequent visits.

#### Scenario: Build assets cached on install

- **WHEN** the service worker installs
- **THEN** it SHALL precache all SvelteKit build output files (JS, CSS, HTML shell)
- **AND** it SHALL precache static assets (icons, manifest, favicon)

#### Scenario: Cache-first for precached assets

- **WHEN** the browser requests a precached asset
- **THEN** the service worker SHALL respond from cache
- **AND** it SHALL fall back to network only if the cache entry is missing

#### Scenario: Non-precached requests pass through

- **WHEN** the browser requests a URL not in the precache set (e.g., `/api/*`, `/uploads/*`)
- **THEN** the service worker SHALL NOT intercept or cache the request
- **AND** the request SHALL go directly to the network

### Requirement: Offline fallback page

The service worker SHALL serve a branded offline fallback page when a navigation request fails and no cached response exists.

#### Scenario: Offline fallback served on navigation failure

- **WHEN** a navigation request fails due to network unavailability
- **AND** no cached response exists for the requested URL
- **THEN** the service worker SHALL respond with a precached offline fallback page

#### Scenario: Offline fallback not used for cached pages

- **WHEN** a navigation request fails due to network unavailability
- **AND** a cached response exists for the requested URL
- **THEN** the service worker SHALL respond with the cached response instead of the offline fallback

### Requirement: Stale cache cleanup on update

The service worker SHALL remove outdated caches when a new version activates.

#### Scenario: Old caches deleted on activation

- **WHEN** a new service worker version activates
- **THEN** it SHALL delete any caches that do not belong to the current version

#### Scenario: New worker activates immediately

- **WHEN** a new service worker version finishes installing
- **THEN** it SHALL call `skipWaiting()` to activate immediately without waiting for existing clients to close

### Requirement: Update notification

The app SHALL notify the user when a new service worker version has activated so they can reload to get the latest version.

#### Scenario: Update prompt shown on controller change

- **WHEN** a new service worker takes control of the page
- **THEN** the app SHALL display a notification prompting the user to reload

#### Scenario: No update prompt on first registration

- **WHEN** the service worker is registered for the first time (no previous controller)
- **THEN** the app SHALL NOT display an update notification
