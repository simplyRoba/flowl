## MODIFIED Requirements

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

## ADDED Requirements

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
