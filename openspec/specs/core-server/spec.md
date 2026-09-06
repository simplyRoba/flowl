## Purpose

HTTP server lifecycle, health endpoint, browser UI document and asset serving from the deployed application artifact, managed media serving, and graceful shutdown.

## Requirements

### Requirement: HTTP Server Startup

The application SHALL listen for HTTP requests on the port specified by `FLOWL_PORT` (default `4100`).

#### Scenario: Server starts on default port

- **WHEN** the application starts without `FLOWL_PORT` set
- **THEN** the server listens on port `4100`

#### Scenario: Server starts on custom port

- **WHEN** the application starts with `FLOWL_PORT=3000`
- **THEN** the server listens on port `3000`

### Requirement: Health Endpoint

The server SHALL expose a `GET /health` endpoint that checks durable application-data availability and returns a JSON body indicating service health.

#### Scenario: Health check succeeds

- **WHEN** a GET request is made to `/health`
- **AND** durable application data is available
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{"status": "ok"}`

#### Scenario: Health check fails

- **WHEN** a GET request is made to `/health`
- **AND** durable application data is unavailable
- **THEN** the server responds with HTTP 503
- **AND** the response body is `{"status": "unhealthy"}`

### Requirement: Browser UI document and asset serving

The server SHALL make the browser UI available from the deployed application artifact without requiring a separately deployed frontend. It SHALL serve the UI index document and app assets with their correct MIME types. A browser UI route outside the reserved `/api` and `/uploads` namespaces that does not match an app asset SHALL receive the UI `index.html` document for client-side routing.

#### Scenario: Root path serves the UI document

- **WHEN** a GET request is made to `/`
- **THEN** the server responds with the UI `index.html` document

#### Scenario: App asset served

- **WHEN** a GET request is made to the exact path of an app asset included in the deployed application artifact
- **THEN** the server responds with the asset contents and correct MIME type

#### Scenario: Unknown browser route falls back to the UI document

- **WHEN** a GET request is made to a browser UI path outside the `/api` and `/uploads` namespaces that does not match an app asset
- **THEN** the server responds with the UI `index.html` document for client-side routing

#### Scenario: API namespace takes precedence

- **WHEN** a request is made to `/api` or a path under `/api/*`
- **THEN** the request is handled as an API request
- **AND** it does not fall back to the UI document

#### Scenario: Managed-media namespace takes precedence

- **WHEN** a request is made to `/uploads` or a path under `/uploads/*`
- **THEN** the request is handled as a managed-media request
- **AND** it does not fall back to the UI document

### Requirement: Structured Logging

The application SHALL emit structured logs filtered according to `FLOWL_LOG_LEVEL` (default `info`).

#### Scenario: Default log level

- **WHEN** the application starts without `FLOWL_LOG_LEVEL` set
- **THEN** log output is filtered at `info` level

#### Scenario: Custom log level

- **WHEN** the application starts with `FLOWL_LOG_LEVEL=debug`
- **THEN** log output includes `debug` level messages

### Requirement: Managed media serving

The server SHALL serve managed media originals and renditions at `/uploads/*`, as defined by `core-image-store`.

#### Scenario: Managed media served

- **WHEN** a GET request is made to `/uploads/abc.jpg`
- **AND** the managed media exists
- **THEN** the server responds with the file contents

### Requirement: Graceful Shutdown

The server SHALL shut down gracefully on SIGTERM or SIGINT, closing open connections before exiting.

#### Scenario: SIGTERM received

- **WHEN** the process receives SIGTERM
- **THEN** the server stops accepting new connections
- **AND** the process exits with code 0

### Requirement: Authentication-aware SPA document access

When authentication is enabled, the server SHALL keep `/login` and exact non-data resources required for login/PWA rendering public while requiring an authenticated session before serving `index.html` for root, normal application routes, or unknown browser UI document fallbacks. Authentication-disabled browser UI document and asset behavior SHALL remain as defined by the canonical Browser UI document and asset serving requirement.

#### Scenario: Public login document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser requests `/login`
- **THEN** the server responds with the SPA document needed for the public login route

#### Scenario: Public non-document resources

- **WHEN** authentication is enabled
- **AND** an unauthenticated client requests an exact app asset, service worker, manifest, icon, favicon, or offline page required for login/PWA operation
- **THEN** the server serves that exact resource without authentication
- **AND** does not make `index.html` or an unknown browser UI document fallback public

#### Scenario: Protected SPA document

- **WHEN** authentication is enabled
- **AND** an unauthenticated browser navigates to `/`, `/index.html`, a normal application route, or an unknown document fallback
- **THEN** the server redirects to `/login?return_to=<safe-local-path-and-query>`
- **AND** does not serve the protected SPA document at that target

### Requirement: Authentication-aware upload access

When authentication is enabled, every request in the `/uploads` namespace, including the exact `/uploads` path and its descendants, SHALL require a valid authenticated session. Authentication-disabled media serving SHALL remain as defined by the canonical Managed media serving requirement.

#### Scenario: Authenticated uploaded file served

- **WHEN** authentication is enabled
- **AND** a GET request with a valid session is made to `/uploads/abc.jpg`
- **AND** the managed media exists
- **THEN** the server responds with the file contents

#### Scenario: Unauthenticated upload is protected

- **WHEN** authentication is enabled
- **AND** a request without a valid session is made to `/uploads/abc.jpg`
- **THEN** the server responds with HTTP 401
- **AND** does not return file bytes or a login/provider redirect
