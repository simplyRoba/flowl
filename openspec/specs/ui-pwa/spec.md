## Requirements

### Requirement: Web app manifest

The app SHALL provide a web app manifest at `/manifest.json` that makes it installable on mobile devices.

#### Scenario: Manifest is served

- **WHEN** a browser requests `/manifest.json`
- **THEN** a valid JSON manifest SHALL be returned
- **AND** it SHALL include `name`, `short_name`, `start_url`, `display`, `theme_color`, `background_color`, and `icons`

#### Scenario: Display mode

- **WHEN** the app is installed via "Add to Home Screen"
- **THEN** it SHALL launch in `standalone` mode without browser chrome

#### Scenario: Theme colors match app

- **WHEN** the manifest is loaded
- **THEN** `theme_color` SHALL be `#FAF6F1` (light background)
- **AND** `background_color` SHALL be `#FAF6F1`

### Requirement: PWA icons

The app SHALL provide PNG icons in the sizes required for installability.

#### Scenario: Required icon sizes

- **WHEN** the manifest `icons` array is read
- **THEN** it SHALL include at least a 192x192 icon with `purpose: "any"`
- **AND** a 512x512 icon with `purpose: "any"`

### Requirement: Manifest link tag

The HTML document SHALL reference the manifest.

#### Scenario: Link tag present

- **WHEN** the HTML document is loaded
- **THEN** a `<link rel="manifest" href="/manifest.json">` tag SHALL be present in the `<head>`

### Requirement: Pull-to-refresh in standalone PWA mode

The app SHALL provide a custom pull-to-refresh gesture on allowlisted browse routes when running in standalone PWA mode on touch devices.

#### Scenario: Touch tablet in standalone mode is eligible

- **WHEN** the app is running in standalone PWA mode on a touch-capable tablet
- **AND** the user is on an allowlisted route
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on dashboard

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on care journal

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/care-journal`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on settings

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/settings`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on plant detail

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/42`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

### Requirement: Pull-to-refresh route exclusions

The app SHALL NOT provide the custom pull-to-refresh gesture on non-allowlisted routes.

#### Scenario: New plant route excluded

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/new`
- **THEN** pulling down SHALL NOT arm the custom refresh gesture

#### Scenario: Edit plant route excluded

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/42/edit`
- **THEN** pulling down SHALL NOT arm the custom refresh gesture

### Requirement: Pull-to-refresh reload behavior

Once armed on an allowlisted route, the gesture SHALL trigger a full reload of the current route when the user releases beyond the refresh threshold.

#### Scenario: Release beyond threshold reloads page

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on an allowlisted route
- **AND** the document is scrolled to the top
- **AND** the user pulls beyond the refresh threshold and releases
- **THEN** the app SHALL perform a full reload of the current route

#### Scenario: Release before threshold does not reload page

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on an allowlisted route
- **AND** the document is scrolled to the top
- **AND** the user releases before reaching the refresh threshold
- **THEN** the app SHALL cancel the gesture without reloading the route

### Requirement: Pull-to-refresh feedback and safety gates

The app SHALL provide visible feedback while the gesture is active and SHALL suppress the gesture when the browsing context is not safe for refresh.

#### Scenario: Feedback shown during pull

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user pulls down from the top on an allowlisted route
- **THEN** the app SHALL display a visible pull-to-refresh indicator

#### Scenario: Brief refreshing state shown after release

- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user releases beyond the refresh threshold on an allowlisted route
- **THEN** the pull-to-refresh indicator SHALL transition into a brief refreshing state before the page reload handoff

#### Scenario: Gesture ignored away from top of page

- **WHEN** the user is on an allowlisted route
- **AND** the document is not scrolled to the top
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm

#### Scenario: Gesture suppressed while transient overlay is open

- **WHEN** the user is on `/plants/42`
- **AND** a transient overlay such as a modal dialog, lightbox, chat drawer, or inline care entry flow is open
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm

#### Scenario: Gesture unavailable outside standalone mode

- **WHEN** the app is running in a normal browser tab instead of standalone PWA mode
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm

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

- **WHEN** the browser requests a URL not in the precache set and not matching a cacheable API endpoint or thumbnail pattern
- **THEN** the service worker SHALL NOT intercept or cache the request
- **AND** the request SHALL go directly to the network

### Requirement: API response caching

The service worker SHALL cache GET responses for plant-related API endpoints using a network-first strategy with stale fallback.

#### Scenario: Cacheable API endpoints

- **WHEN** the browser makes a GET request to any of the following endpoints: `/api/plants`, `/api/plants/{id}`, `/api/plants/{id}/care`, `/api/stats`, `/api/locations`
- **THEN** the service worker SHALL intercept the request

#### Scenario: Network-first for API requests

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network is available
- **THEN** the service worker SHALL fetch the response from the network
- **AND** it SHALL clone the response and store it in the API cache
- **AND** it SHALL return the network response to the caller

#### Scenario: Stale fallback when offline

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network request fails
- **AND** a cached response exists for the request URL
- **THEN** the service worker SHALL return the cached response

#### Scenario: No cache and no network

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network request fails
- **AND** no cached response exists
- **THEN** the service worker SHALL let the fetch fail naturally so the calling code receives the error

#### Scenario: Non-cacheable API requests pass through

- **WHEN** the browser makes a GET request to an API endpoint not in the cacheable list (e.g., `/api/care`, `/api/settings`, `/api/mqtt/status`, `/api/ai/status`, `/api/info`)
- **THEN** the service worker SHALL NOT intercept or cache the request

#### Scenario: Non-GET requests are never cached

- **WHEN** the browser makes a POST, PUT, or DELETE request to any endpoint
- **THEN** the service worker SHALL NOT intercept or cache the request

### Requirement: Thumbnail image caching

The service worker SHALL cache thumbnail images using a cache-first strategy.

#### Scenario: Thumbnail request cached

- **WHEN** the browser requests a URL matching the thumbnail pattern (`/uploads/*_200.jpg`, `/uploads/*_600.jpg`, or `/uploads/*_1000.jpg`)
- **THEN** the service worker SHALL check the API cache for a stored response
- **AND** if cached, it SHALL return the cached response without a network request

#### Scenario: Thumbnail cache miss

- **WHEN** the browser requests a thumbnail URL
- **AND** no cached response exists
- **THEN** the service worker SHALL fetch from the network
- **AND** it SHALL store the response in the API cache for future requests

#### Scenario: Full-size images not cached

- **WHEN** the browser requests an upload URL that does not match the thumbnail pattern (e.g., `/uploads/abc123.png`)
- **THEN** the service worker SHALL NOT intercept or cache the request

### Requirement: API cache lifecycle

The service worker SHALL maintain the API cache separately from the static asset cache, with version-based cleanup.

#### Scenario: Separate cache name

- **WHEN** the service worker stores an API response or thumbnail
- **THEN** it SHALL use a cache named `flowl-api-{version}` where `{version}` matches the SvelteKit build version

#### Scenario: Old API caches cleaned on activation

- **WHEN** a new service worker version activates
- **THEN** it SHALL delete any caches that do not belong to the current version
- **AND** this SHALL include both old static asset caches and old API caches

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
