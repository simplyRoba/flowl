## Why

The project has CI/CD and a Dockerfile but no application code. Before any features can be built, the foundational runtime needs to exist: an HTTP server, a database, a frontend shell, and MQTT connectivity. This change establishes the skeleton that all subsequent phases build on.

## What Changes

- Add Axum HTTP server with a `/health` endpoint on a configurable port (`FLOWL_PORT`, default 8080).
- Add SQLite database via sqlx with migration infrastructure and an initial empty migration.
- Scaffold a SvelteKit frontend project and embed the build output in the Rust binary via `rust-embed`.
- Add an MQTT client (`rumqttc`) that connects to a configurable Mosquitto broker on startup.
- Add environment-based configuration for port, database path, and MQTT settings.
- Add structured logging with `tracing` and `tracing-subscriber`.
- Update `Cargo.toml` with all phase-1 dependencies.

## Capabilities

### New Capabilities

- `core/server`: Axum HTTP server lifecycle, health endpoint, static file serving for the embedded SvelteKit SPA.
- `core/database`: SQLite connection pool via sqlx, migration runner, database file configuration.
- `core/mqtt`: MQTT client connection to Mosquitto, graceful connect/disconnect, configuration via env vars.
- `ui/shell`: SvelteKit project scaffold with build pipeline, embedded in the Rust binary and served as the default route.

### Modified Capabilities

- `core/foundation`: The Dockerfile health check (`/health`) now has a backing implementation.

## Impact

- `Cargo.toml`: New dependencies (axum, tokio, sqlx, rumqttc, rust-embed, tower-http, serde, tracing).
- `src/main.rs`: Replaced with application bootstrap (server, DB, MQTT, config).
- `src/`: New modules for config, database, mqtt, and http routing.
- `ui/`: New SvelteKit project directory with build config.
- `migrations/`: sqlx migration directory.
- `build.rs`: Build script to compile SvelteKit before embedding.
- `Dockerfile`: No changes needed (already expects the binary at `/usr/local/bin/flowl`).
