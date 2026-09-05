## Purpose

Progressive Web App capability: web app manifest, PWA icons, service worker registration, asset/API/thumbnail caching, offline fallback, cache lifecycle and update notifications, and pull-to-refresh in standalone mode.

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
- **THEN** the app SHALL activate its PWA offline support

#### Scenario: No service worker in development

- **WHEN** the app loads in a development build
- **THEN** no service worker SHALL be registered

### Requirement: Static asset precaching

The service worker SHALL precache all static build assets so they are available without network access on subsequent visits.

#### Scenario: Build assets cached on install

- **WHEN** the service worker installs
- **THEN** it SHALL precache all packaged application scripts, styles, and the HTML shell
- **AND** it SHALL precache static assets (icons, manifest, favicon)

#### Scenario: Cache-first for precached assets

- **WHEN** the browser requests a precached asset
- **THEN** the service worker SHALL respond from cache
- **AND** it SHALL fall back to network only if the cache entry is missing

#### Scenario: Non-precached requests pass through

- **WHEN** the browser requests a URL not in the precache set and not matching a cacheable API endpoint or a canonical rendition URL defined by `core-image-store`
- **THEN** the service worker SHALL NOT intercept or cache the request
- **AND** the request SHALL go directly to the network

### Requirement: API response caching

For cacheable plant-related GET endpoints, the service worker SHALL prefer a fresh network response and use a stored response only when the network request fails without an HTTP response.

#### Scenario: Cacheable API endpoints

- **WHEN** the browser makes a GET request to any of the following endpoints: `/api/plants`, `/api/plants/{id}`, `/api/plants/{id}/care`, `/api/stats`, `/api/locations`
- **THEN** the service worker SHALL intercept the request

#### Scenario: Network-first for API requests

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network is available
- **THEN** the service worker SHALL return the network response to the caller
- **AND** when authentication is disabled, retain the received response for later offline fallback
- **AND** when authentication is enabled, retain it only under the authentication-safe policy below

#### Scenario: Stale fallback when offline

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network request rejects without an HTTP response
- **AND** a cached response exists for the request URL
- **THEN** the service worker SHALL return the cached response

#### Scenario: No cache and no network

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network request rejects without an HTTP response
- **AND** no cached response exists
- **THEN** the service worker SHALL let the fetch fail naturally so the calling code receives the error

#### Scenario: Non-cacheable API requests pass through

- **WHEN** the browser makes a GET request to an API endpoint not in the cacheable list (e.g., `/api/care`, `/api/settings`, `/api/mqtt/status`, `/api/ai/status`, `/api/info`)
- **THEN** the service worker SHALL NOT intercept or cache the request

#### Scenario: Non-GET requests are never cached

- **WHEN** the browser makes a POST, PUT, or DELETE request to any endpoint
- **THEN** the service worker SHALL NOT intercept or cache the request

### Requirement: Thumbnail image caching

When authentication is disabled, the service worker SHALL prefer a stored thumbnail response and fetch the thumbnail only when no stored response exists.

#### Scenario: Thumbnail request cached

- **WHEN** the browser requests a canonical rendition URL defined by `core-image-store`
- **THEN** the service worker SHALL check for a stored response
- **AND** if cached, it SHALL return the cached response without a network request

#### Scenario: Thumbnail cache miss

- **WHEN** the browser requests a thumbnail URL
- **AND** no stored response exists
- **THEN** the service worker SHALL fetch the response from the network
- **AND** retain that response for future requests

#### Scenario: Full-size images not cached

- **WHEN** the browser requests an original full-size upload URL rather than a canonical rendition URL defined by `core-image-store`
- **THEN** the service worker SHALL NOT intercept or cache the request

### Requirement: API cache lifecycle

The service worker SHALL keep current application assets, API data, and thumbnail photos isolated from one another so each category can follow its required freshness, privacy, and cleanup behavior.

#### Scenario: Cached content categories remain separate

- **WHEN** the service worker stores application assets, cacheable API data, or thumbnails
- **THEN** each category SHALL remain isolated so its own freshness, privacy, and cleanup policy can be applied

#### Scenario: Current cache set retained on activation

- **WHEN** a service worker version activates
- **THEN** it SHALL retain current application assets, API data, and thumbnails

#### Scenario: Old API caches cleaned on activation

- **WHEN** a service worker version activates
- **THEN** it SHALL delete obsolete versions of the static, API data, and thumbnail caches
- **AND** current-version static, API data, and thumbnail caches SHALL remain available

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

When a new application version activates, the service worker SHALL remove obsolete cached application content while retaining content belonging to the current version.

#### Scenario: Old caches deleted on activation

- **WHEN** a new service worker version activates
- **THEN** it SHALL retain the current static, API data, and thumbnail caches
- **AND** it SHALL delete obsolete versions of those caches and other obsolete Flowl runtime caches

#### Scenario: New worker activates immediately

- **WHEN** a new service worker version finishes installing
- **THEN** it SHALL become active without requiring existing app windows to close first

### Requirement: Update notification

The app SHALL notify the user when a new service worker version has activated so they can reload to get the latest version.

#### Scenario: Update prompt shown when an update becomes active

- **WHEN** a newly installed application version becomes active
- **THEN** the app SHALL display a notification prompting the user to reload

#### Scenario: No update prompt on first registration

- **WHEN** PWA offline support is activated for the first time
- **THEN** the app SHALL NOT display an update notification

### Requirement: Authentication-safe API caching

When authentication is enabled, the service worker SHALL cache only HTTP 200 responses for the canonical cacheable API allowlist. It SHALL return every received unsuccessful HTTP response directly without caching it or substituting stale data; canonical stale fallback SHALL occur only when the network request rejects without an HTTP response. Authentication-disabled API caching SHALL remain unchanged.

#### Scenario: Successful protected API response is cached

- **WHEN** authentication is enabled
- **AND** a canonical cacheable API request returns HTTP 200
- **THEN** the service worker caches and returns the fresh response

#### Scenario: Authentication-required response is not cached or hidden

- **WHEN** authentication is enabled
- **AND** a canonical cacheable API request returns HTTP 401 with code `AUTHENTICATION_REQUIRED`
- **THEN** the service worker returns that response to the caller
- **AND** does not cache it or replace it with stale data

#### Scenario: Other unsuccessful response is not cached or hidden

- **WHEN** authentication is enabled
- **AND** a canonical cacheable API request returns any non-200 HTTP response
- **THEN** the service worker returns that response to the caller
- **AND** does not cache it or use stale fallback

#### Scenario: Protected API transport failure uses canonical stale fallback

- **WHEN** authentication is enabled
- **AND** a canonical cacheable API network request rejects without an HTTP response
- **THEN** the canonical cached-response or natural-fetch-failure behavior remains in effect

### Requirement: Authentication-aware thumbnail caching

When authentication is enabled, canonical thumbnail URLs SHALL use network-first behavior, cache only HTTP 200 image responses, return received authentication/error/redirect responses without stale substitution, and use a cached thumbnail only when the network rejects without an HTTP response. Authentication-disabled thumbnail cache-first behavior and canonical full-size-image behavior SHALL remain unchanged.

#### Scenario: Fresh authenticated thumbnail is cached

- **WHEN** authentication is enabled
- **AND** a protected thumbnail request returns HTTP 200
- **THEN** the service worker caches and returns the fresh image response

#### Scenario: Online authentication failure is not hidden by stale thumbnail

- **WHEN** authentication is enabled
- **AND** a cached thumbnail exists
- **AND** the online request returns HTTP 401 or a redirect
- **THEN** the service worker returns the network response
- **AND** does not return or overwrite the stale thumbnail

#### Scenario: Offline thumbnail uses stale fallback

- **WHEN** authentication is enabled
- **AND** a cached thumbnail exists
- **AND** the network request rejects without an HTTP response
- **THEN** the service worker returns the cached thumbnail

### Requirement: Authentication responses are network-only

The service worker SHALL always obtain `/auth/*` requests and responses, login results, authorization redirects, callbacks, logout responses, and authentication-required errors from the network and SHALL NOT store them in any PWA-managed cache or persistent offline store.

#### Scenario: Auth endpoint requested

- **WHEN** a request targets any `/auth/*` endpoint
- **THEN** it is obtained from the network and is not stored in any PWA-managed cache or persistent offline store

#### Scenario: Authentication redirect received

- **WHEN** a response redirects to `/login` or an OIDC provider
- **THEN** the response is not stored as application data

### Requirement: Authentication-aware navigation fallback

When authentication is enabled and the network responds, the backend response SHALL take precedence over cached application content for navigation to normal application routes. Only when navigation fails without an HTTP response SHALL the service worker use the cached application shell or branded offline fallback. `/auth/*` navigation SHALL remain network-only, and public `/login` resources SHALL remain available.

#### Scenario: Online unauthenticated navigation reaches backend

- **WHEN** authentication is enabled
- **AND** `index.html` or another application shell is precached
- **AND** an unauthenticated browser navigates online to a normal application route
- **THEN** the service worker returns the backend login redirect instead of cached application content

#### Scenario: Temporary outage preserves canonical navigation fallback

- **WHEN** authentication is enabled
- **AND** navigation to a normal application route rejects without an HTTP response
- **THEN** the canonical cached-shell or branded-offline-page fallback remains in effect

### Requirement: Explicit logout protected-cache purge

Explicit logout SHALL request removal of protected API, photo, and application-navigation data from both current and obsolete offline content and SHALL wait for confirmation that removal completed before logout navigation. It SHALL retain public login/PWA resources, update metadata, and local theme/locale preferences. Neither session expiry nor ordinary network loss SHALL trigger this removal.

#### Scenario: Explicit logout purges protected content

- **WHEN** explicit logout begins
- **THEN** the frontend requests removal of protected API, photo, and application-navigation offline content
- **AND** logout navigation waits until the offline layer confirms that removal completed

#### Scenario: Public login resources survive logout

- **WHEN** explicit logout purge completes
- **THEN** `/login`, manifest, icons, offline resources, theme/locale preferences, and service-worker update metadata remain available

#### Scenario: Network loss retains protected offline data

- **WHEN** an ordinary protected request rejects without an HTTP response
- **THEN** the service worker does not purge protected caches
- **AND** existing offline data remains available

#### Scenario: Session expiry retains protected offline data

- **WHEN** an online request receives `401 AUTHENTICATION_REQUIRED` because the authenticated session expired
- **THEN** the service worker does not purge protected caches merely because of expiry
- **AND** the response reaches the frontend expiry handler

### Requirement: Authoritative authentication state for offline content

Whenever the service-worker global starts or restarts, including after eviction and reactivation, it SHALL treat the authentication state as unconfirmed and apply authentication-enabled fail-closed rules to cached application content. Authentication-disabled behavior SHALL be established only by a fresh, successful, non-redirected `/auth/config` response obtained directly from the backend while bypassing all caches; confirmation from an earlier service-worker global lifetime, cached content, or browser-reported state SHALL NOT establish it. Before using authentication-disabled behavior for navigation, API data, or thumbnails, the service worker SHALL recheck the backend configuration so a change to enabled authentication is applied first. When disabled authentication was confirmed during the current service-worker global lifetime and the recheck fails solely because the network is unavailable, the existing authentication-disabled offline behavior SHALL remain available.

#### Scenario: Unconfirmed authentication state fails closed

- **WHEN** the service-worker global starts or restarts, including after eviction and reactivation
- **AND** a fresh `/auth/config` response cannot be obtained
- **THEN** protected fail-closed navigation, API, and thumbnail behavior is used
- **AND** authentication-disabled cache-first behavior is not used

#### Scenario: Backend confirms authentication is disabled

- **WHEN** a fresh non-redirected `/auth/config` response obtained directly from the backend while bypassing all caches reports `enabled: false`
- **THEN** the canonical authentication-disabled cache behavior becomes available

#### Scenario: Authentication config is obtained directly

- **WHEN** the PWA determines or rechecks authentication state
- **THEN** it obtains `/auth/config` directly from the backend while bypassing all caches and does not retain the response
- **AND** cached application content cannot establish that authentication is disabled

#### Scenario: Disabled-to-enabled transition is observed

- **WHEN** the backend previously confirmed that authentication was disabled
- **AND** the backend configuration changes to `enabled: true`
- **AND** a navigation, cacheable API, or thumbnail request could expose cached application data
- **THEN** the PWA confirms the current backend configuration before using authentication-disabled behavior
- **AND** applies authentication-enabled protection before handling that application request

#### Scenario: Browser state cannot establish disabled authentication

- **WHEN** browser-side state reports that authentication is disabled without fresh backend confirmation
- **THEN** authentication-disabled cache behavior is not enabled from that state

#### Scenario: Previously confirmed disabled authentication remains usable offline

- **WHEN** the backend confirmed that authentication was disabled during the current service-worker global lifetime
- **AND** a later configuration request fails solely because the network is unavailable
- **THEN** existing authentication-disabled offline fallback remains available
