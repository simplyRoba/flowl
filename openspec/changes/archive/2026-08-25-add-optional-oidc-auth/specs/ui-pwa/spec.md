## ADDED Requirements

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

The service worker SHALL never place `/auth/*` requests or responses, login result responses, authorization redirects, callback responses, logout responses, or authentication-required error responses in offline application caches.

#### Scenario: Auth endpoint requested

- **WHEN** a request targets any `/auth/*` endpoint
- **THEN** it goes to the network without application-cache lookup or insertion

#### Scenario: Authentication redirect received

- **WHEN** a response redirects to `/login` or an OIDC provider
- **THEN** the response is not stored as application data

### Requirement: Authentication-aware navigation fallback

When authentication is enabled and the network responds, navigation to a normal application route SHALL reach the backend before any static/precache or runtime navigation lookup, including cached `index.html`. Only when navigation rejects without an HTTP response SHALL the service worker use the canonical cached-shell or branded-offline-page fallback. `/auth/*` navigation SHALL remain network-only, and public `/login` resources SHALL remain available.

#### Scenario: Online unauthenticated navigation reaches backend

- **WHEN** authentication is enabled
- **AND** `index.html` or another application shell is precached
- **AND** an unauthenticated browser navigates online to a normal application route
- **THEN** the service worker contacts the backend before every cache lookup
- **AND** returns the backend login redirect

#### Scenario: Temporary outage preserves canonical navigation fallback

- **WHEN** authentication is enabled
- **AND** navigation to a normal application route rejects without an HTTP response
- **THEN** the canonical cached-shell or branded-offline-page fallback remains in effect

### Requirement: Explicit logout protected-cache purge

The service worker SHALL support an acknowledged explicit-logout purge that deletes protected API, photo, and runtime application-navigation data across current and obsolete Flowl cache versions. It SHALL retain public login/PWA resources, service-worker version metadata, and local theme/locale preferences. Neither session expiry nor ordinary network loss SHALL trigger this purge.

#### Scenario: Explicit logout purges protected content

- **WHEN** the frontend sends the explicit logout purge message
- **THEN** the service worker deletes protected API, photo, and runtime application-navigation cache entries
- **AND** acknowledges completion before logout navigation proceeds

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

### Requirement: Authoritative service-worker authentication mode

A newly started service worker SHALL begin with authentication mode unknown. Unknown mode SHALL use authentication-enabled protected, fail-closed policies. Disabled mode SHALL be established only by a fresh, successful, non-redirected, network-only `/auth/config` response reporting `enabled: false`; `/auth/config` SHALL never be satisfied from an application or offline cache. A client message MAY tighten the worker to enabled mode or invalidate its current mode, but SHALL NOT establish disabled mode. While disabled, the worker SHALL recheck authoritative backend configuration before navigation, API, or thumbnail policies capable of exposing cached application data so a disabled-to-enabled backend change is observed. If disabled mode was previously established and that fresh configuration request rejects solely because the network is unavailable, existing authentication-disabled offline behavior SHALL remain available. A configuration failure while mode is unknown SHALL NOT establish or assume disabled mode.

#### Scenario: Newly started worker fails closed while mode is unknown

- **WHEN** a service worker starts or restarts without an authoritative authentication mode
- **AND** fresh `/auth/config` cannot establish the current mode
- **THEN** the worker keeps its mode unknown
- **AND** uses protected fail-closed navigation, API, and thumbnail policies
- **AND** does not use disabled-mode cache-first behavior

#### Scenario: Worker restart establishes disabled mode from the backend

- **WHEN** a newly restarted worker has unknown authentication mode
- **AND** a fresh network-only `/auth/config` response reports `enabled: false`
- **THEN** the worker establishes disabled mode
- **AND** preserves the canonical authentication-disabled cache behavior

#### Scenario: Authentication config is network-only

- **WHEN** the worker determines or rechecks authentication mode
- **THEN** it requests `/auth/config` from the network without application-cache lookup or insertion
- **AND** a cached application or offline response cannot establish disabled mode

#### Scenario: Disabled-to-enabled transition is observed

- **WHEN** the worker previously established disabled mode
- **AND** the backend configuration changes to `enabled: true`
- **AND** a navigation, cacheable API, or thumbnail request could expose cached application data
- **THEN** the worker rechecks `/auth/config` before applying the disabled-mode policy
- **AND** changes to enabled protected policies before handling that application request

#### Scenario: Client cannot establish disabled mode

- **WHEN** a client message reports that authentication is disabled
- **THEN** the worker may invalidate its current mode and request fresh backend configuration
- **AND** does not establish disabled mode from the client message alone

#### Scenario: Previously established disabled mode remains usable offline

- **WHEN** the worker previously established disabled mode from a fresh backend response
- **AND** a later authoritative configuration request rejects solely because the network is unavailable
- **THEN** the worker retains disabled mode for that request
- **AND** existing authentication-disabled offline fallback remains available

## MODIFIED Requirements

### Requirement: API response caching

The service worker SHALL cache GET responses for plant-related API endpoints in the versioned API data cache `flowl-api-{version}` using a network-first strategy with stale fallback.

#### Scenario: Cacheable API endpoints

- **WHEN** the browser makes a GET request to any of the following endpoints: `/api/plants`, `/api/plants/{id}`, `/api/plants/{id}/care`, `/api/stats`, `/api/locations`
- **THEN** the service worker SHALL intercept the request

#### Scenario: Network-first for API requests

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network is available
- **THEN** the service worker SHALL fetch the response from the network
- **AND** it SHALL clone the response and store it in the `flowl-api-{version}` API data cache
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

The service worker SHALL cache thumbnail images in the versioned photo cache `flowl-photo-{version}` using a cache-first strategy when authentication is disabled.

#### Scenario: Thumbnail request cached

- **WHEN** the browser requests a URL matching the thumbnail pattern (`/uploads/*_200.jpg`, `/uploads/*_600.jpg`, or `/uploads/*_1000.jpg`)
- **THEN** the service worker SHALL check the photo cache for a stored response
- **AND** if cached, it SHALL return the cached response without a network request

#### Scenario: Thumbnail cache miss

- **WHEN** the browser requests a thumbnail URL
- **AND** no cached response exists in the photo cache
- **THEN** the service worker SHALL fetch from the network
- **AND** it SHALL store the response in the `flowl-photo-{version}` photo cache for future requests

#### Scenario: Full-size images not cached

- **WHEN** the browser requests an upload URL that does not match the thumbnail pattern (e.g., `/uploads/abc123.png`)
- **THEN** the service worker SHALL NOT intercept or cache the request

### Requirement: API cache lifecycle

The service worker SHALL maintain separate versioned caches for static application assets, API data, and thumbnail photos. Static assets SHALL use `flowl-cache-{version}`, cacheable API data SHALL use `flowl-api-{version}`, and thumbnails SHALL use `flowl-photo-{version}`, where each `{version}` matches the SvelteKit build version.

#### Scenario: Separate cache name

- **WHEN** the service worker stores static assets, cacheable API data, or thumbnails
- **THEN** static assets SHALL use `flowl-cache-{version}`
- **AND** cacheable API data SHALL use `flowl-api-{version}`
- **AND** thumbnails SHALL use `flowl-photo-{version}`

#### Scenario: Current cache set retained on activation

- **WHEN** a service worker version activates
- **THEN** it SHALL retain the current `flowl-cache-{version}` static cache
- **AND** it SHALL retain the current `flowl-api-{version}` API data cache
- **AND** it SHALL retain the current `flowl-photo-{version}` thumbnail cache

#### Scenario: Old API caches cleaned on activation

- **WHEN** a service worker version activates
- **THEN** it SHALL delete obsolete versions of the static, API data, and thumbnail caches
- **AND** current-version static, API data, and thumbnail caches SHALL remain available

### Requirement: Stale cache cleanup on update

The service worker SHALL remove obsolete versioned application caches when a new version activates while retaining the current static `flowl-cache-{version}`, API data `flowl-api-{version}`, and thumbnail `flowl-photo-{version}` caches.

#### Scenario: Old caches deleted on activation

- **WHEN** a new service worker version activates
- **THEN** it SHALL retain the current static, API data, and thumbnail caches
- **AND** it SHALL delete obsolete versions of those caches and other obsolete Flowl runtime caches

#### Scenario: New worker activates immediately

- **WHEN** a new service worker version finishes installing
- **THEN** it SHALL call `skipWaiting()` to activate immediately without waiting for existing clients to close
