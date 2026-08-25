## MODIFIED Requirements

### Requirement: API response caching

The service worker SHALL cache only successful HTTP 200 GET responses for the existing plant-related API allowlist using a network-first strategy with stale fallback only when the network request rejects. It SHALL return any received HTTP response, including `401 AUTHENTICATION_REQUIRED`, directly to the caller without caching it or substituting stale application data. Authentication being disabled SHALL NOT change the existing endpoint allowlist or successful-response offline behavior.

#### Scenario: Cacheable API endpoints

- **WHEN** the browser makes a GET request to `/api/plants`, `/api/plants/{id}`, `/api/plants/{id}/care`, `/api/stats`, or `/api/locations`
- **THEN** the service worker SHALL intercept the request

#### Scenario: Network-first for API requests

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network returns HTTP 200
- **THEN** the service worker SHALL clone and store the response in the protected API cache
- **AND** return the network response to the caller

#### Scenario: Authentication-required response is not cached or hidden

- **WHEN** the network returns HTTP 401 with code `AUTHENTICATION_REQUIRED` for a cacheable API request
- **THEN** the service worker SHALL return that response to the caller
- **AND** SHALL NOT cache it
- **AND** SHALL NOT replace it with stale cached data

#### Scenario: Other unsuccessful response is not cached or hidden

- **WHEN** the network returns any non-200 HTTP response for a cacheable API request
- **THEN** the service worker SHALL return that response to the caller
- **AND** SHALL NOT cache it or use stale fallback

#### Scenario: Stale fallback when offline

- **WHEN** the service worker intercepts a cacheable API request
- **AND** the network request rejects because no HTTP response was received
- **AND** a cached successful response exists for the request URL
- **THEN** the service worker SHALL return the cached response

#### Scenario: No cache and no network

- **WHEN** a cacheable API network request rejects
- **AND** no cached response exists
- **THEN** the fetch SHALL fail naturally so existing offline handling runs

#### Scenario: Non-cacheable API requests pass through

- **WHEN** the browser makes a GET request to an API endpoint outside the existing allowlist
- **THEN** the service worker SHALL NOT cache the request

#### Scenario: Non-GET requests are never cached

- **WHEN** the browser makes a POST, PUT, or DELETE request to an API endpoint
- **THEN** the service worker SHALL NOT place its request or response in an application cache

### Requirement: Thumbnail image caching

The service worker SHALL use network-first behavior with stale fallback on transport rejection for thumbnail images matching `/uploads/*_200.jpg`, `/uploads/*_600.jpg`, or `/uploads/*_1000.jpg`. It SHALL store only an HTTP 200 image response; an authentication failure, redirect, or other unsuccessful HTTP response SHALL pass through without being cached or replaced by stale data. Full-size originals SHALL remain uncached.

#### Scenario: Thumbnail request cached

- **WHEN** a thumbnail has a cached successful response
- **AND** its network request rejects without an HTTP response
- **THEN** the service worker SHALL return the cached thumbnail

#### Scenario: Thumbnail cache miss

- **WHEN** the protected thumbnail network request returns HTTP 200
- **THEN** the service worker SHALL cache and return the fresh image response
- **AND** SHALL replace any older cached response for that URL

#### Scenario: Online unauthenticated thumbnail with stale cache

- **WHEN** a cached thumbnail exists
- **AND** the online network request returns HTTP 401 or a redirect
- **THEN** the service worker SHALL return the network response
- **AND** SHALL NOT return or overwrite the stale thumbnail

#### Scenario: Full-size images not cached

- **WHEN** an upload URL does not match the thumbnail pattern
- **THEN** the service worker SHALL NOT cache the response

### Requirement: API cache lifecycle

The service worker SHALL maintain protected API, thumbnail, and runtime application-navigation data separately from public static/login resources. It SHALL remove obsolete versioned caches on activation and SHALL support an acknowledged explicit-logout purge that deletes all protected cached data across Flowl cache versions while retaining only resources required for the public login experience, local theme/locale preferences, and service-worker version bookkeeping. Session expiry or an ordinary network failure SHALL NOT trigger this purge.

#### Scenario: Separate cache name

- **WHEN** the service worker stores a successful cacheable API response or thumbnail
- **THEN** it SHALL use the protected `flowl-api-{version}` cache

#### Scenario: Old API caches cleaned on activation

- **WHEN** a new service worker version activates
- **THEN** obsolete static and API caches are deleted according to the existing version lifecycle

#### Scenario: Explicit logout purges protected content

- **WHEN** the frontend sends the explicit logout purge message
- **THEN** the service worker SHALL delete protected API, photo, and runtime application-navigation entries across current and old Flowl cache versions
- **AND** acknowledge completion before logout navigation proceeds

#### Scenario: Public login resources survive logout

- **WHEN** explicit logout purge completes
- **THEN** `/login` can still render using retained public static resources
- **AND** manifest, icons, theme and locale preferences, and service-worker update metadata remain available

#### Scenario: Network loss retains offline data

- **WHEN** an ordinary protected request fails without an HTTP response
- **THEN** the service worker SHALL NOT purge protected caches
- **AND** existing stale offline fallback remains available

#### Scenario: Session expiry retains offline data

- **WHEN** an online request receives `401 AUTHENTICATION_REQUIRED` because the twelve-hour session expired
- **THEN** the service worker SHALL NOT purge existing protected caches merely because of expiry
- **AND** the response still reaches the frontend expiry handler

## ADDED Requirements

### Requirement: Authentication responses are network-only

The service worker SHALL never place `/auth/*` requests or responses, login result responses, authorization redirects, callback responses, logout responses, or authentication-required error responses in offline application caches.

#### Scenario: Auth endpoint requested

- **WHEN** a request targets any `/auth/*` endpoint
- **THEN** it goes to the network without application-cache lookup or insertion

#### Scenario: Authentication redirect received

- **WHEN** a response redirects to `/login` or an OIDC provider
- **THEN** the response is not stored as application data

### Requirement: Authentication-aware navigation fallback

When online, navigation to a normal application route SHALL reach the backend before any static/precache or runtime navigation cache lookup, including a cached `/index.html`, so the backend can enforce the current session. Only when the navigation network request rejects SHALL the service worker preserve existing offline access by falling back to a cached application shell for normal routes when available, otherwise to the branded offline page. `/auth/*` navigation SHALL remain network-only, and `/login` resources SHALL remain available.

#### Scenario: Online unauthenticated navigation reaches backend

- **WHEN** authentication is enabled
- **AND** `/index.html` or another application shell is precached
- **AND** an unauthenticated browser navigates online to a normal application route
- **THEN** the service worker contacts the backend before every cache lookup
- **AND** the backend login redirect is returned

#### Scenario: Temporary outage uses cached application shell

- **WHEN** navigation to a normal application route rejects because of a network outage
- **AND** a cached application shell is available
- **THEN** the service worker returns the cached shell
- **AND** cached protected API and thumbnail data remains usable

#### Scenario: Temporary outage without shell uses offline page

- **WHEN** navigation rejects because of a network outage
- **AND** no cached application shell is available
- **THEN** the service worker returns the precached branded offline page

#### Scenario: Existing offline behavior with auth disabled

- **WHEN** authentication is disabled and the network is unavailable
- **THEN** existing cached application, API, and thumbnail behavior remains functional
