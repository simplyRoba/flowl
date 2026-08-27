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
