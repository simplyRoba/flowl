## Purpose

Axum HTTP server lifecycle, health endpoint, static file serving for the embedded SvelteKit SPA, upload file serving, and graceful shutdown.

## Requirements

### Requirement: HTTP Server Startup

The application SHALL start an Axum HTTP server listening on the port specified by `FLOWL_PORT` (default `8080`).

#### Scenario: Server starts on default port

- **WHEN** the application starts without `FLOWL_PORT` set
- **THEN** the server listens on port `8080`

#### Scenario: Server starts on custom port

- **WHEN** the application starts with `FLOWL_PORT=3000`
- **THEN** the server listens on port `3000`

### Requirement: Health Endpoint

The server SHALL expose a `GET /health` endpoint that returns HTTP 200 with a JSON body indicating the service is healthy.

#### Scenario: Health check succeeds

- **WHEN** a GET request is made to `/health`
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{"status": "ok"}`

### Requirement: SPA Static File Serving

The server SHALL serve the embedded SvelteKit build output as static files. Any request that does not match an API route SHALL fall back to the SPA's `index.html`.

#### Scenario: Root path serves SPA

- **WHEN** a GET request is made to `/`
- **THEN** the server responds with the SvelteKit `index.html`

#### Scenario: Static asset served

- **WHEN** a GET request is made to a path matching an embedded static file (e.g., `/_app/immutable/entry/start.js`)
- **THEN** the server responds with the file contents and correct MIME type

#### Scenario: Unknown path falls back to SPA

- **WHEN** a GET request is made to a path that does not match any API route or static file
- **THEN** the server responds with the SvelteKit `index.html` for client-side routing

#### Scenario: API routes handled by API router

- **WHEN** a GET request is made to a path starting with `/api`
- **THEN** the request is handled by the nested API router
- **AND** does not fall through to the SPA handler

### Requirement: Structured Logging

The application SHALL use `tracing` for structured logging, configured via `FLOWL_LOG_LEVEL` (default `info`).

#### Scenario: Default log level

- **WHEN** the application starts without `FLOWL_LOG_LEVEL` set
- **THEN** log output is filtered at `info` level

#### Scenario: Custom log level

- **WHEN** the application starts with `FLOWL_LOG_LEVEL=debug`
- **THEN** log output includes `debug` level messages

### Requirement: AppState

The server SHALL use an `AppState` struct containing `SqlitePool` and `PathBuf` (upload directory) as the shared state for all routes.

#### Scenario: AppState constructed

- **WHEN** the application starts
- **THEN** `AppState` is created with the database pool and upload directory path
- **AND** the upload directory is created if it does not exist

### Requirement: Upload File Serving

The server SHALL serve files from the upload directory at `/uploads/*` using `tower-http::ServeDir`.

#### Scenario: Uploaded file served

- **WHEN** a GET request is made to `/uploads/abc.jpg`
- **AND** the file exists in the upload directory
- **THEN** the server responds with the file contents

### Requirement: Graceful Shutdown

The server SHALL shut down gracefully on SIGTERM or SIGINT, closing open connections before exiting.

#### Scenario: SIGTERM received

- **WHEN** the process receives SIGTERM
- **THEN** the server stops accepting new connections
- **AND** the process exits with code 0
