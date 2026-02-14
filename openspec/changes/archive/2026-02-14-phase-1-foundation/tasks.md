## 1. Project Dependencies

- [x] 1.1 Add phase-1 dependencies to `Cargo.toml`: axum, tokio, sqlx (sqlite, runtime-tokio), rumqttc, rust-embed, tower-http, serde, serde_json, tracing, tracing-subscriber.

## 2. Configuration

- [x] 2.1 Create `src/config.rs` with a `Config` struct parsing `FLOWL_PORT`, `FLOWL_DB_PATH`, `FLOWL_MQTT_HOST`, `FLOWL_MQTT_PORT`, `FLOWL_MQTT_TOPIC_PREFIX`, and `FLOWL_LOG_LEVEL` from environment variables with defaults.

## 3. Database

- [x] 3.1 Create `src/db.rs` with a function to create a sqlx `SqlitePool`, enabling `create_if_missing` on the configured path.
- [x] 3.2 Create an initial empty migration in `migrations/` to bootstrap the migrations table.
- [x] 3.3 Run `sqlx::migrate!()` at startup before the server starts accepting requests.

## 4. MQTT

- [x] 4.1 Create `src/mqtt.rs` with a function that creates a `rumqttc` async client, connects to the configured broker, and spawns a background task to handle the event loop.
- [x] 4.2 Handle connection errors gracefully: log warnings but do not block server startup.
- [x] 4.3 Disconnect the MQTT client cleanly on application shutdown.

## 5. HTTP Server

- [x] 5.1 Create `src/server.rs` with the Axum router: `GET /health` returning `{"status": "ok"}`.
- [x] 5.2 Create `src/embedded.rs` with a `rust-embed` asset struct pointing at `ui/build/` and a fallback handler serving `index.html` for SPA routing.
- [x] 5.3 Mount the embedded static files and SPA fallback on the Axum router.
- [x] 5.4 Implement graceful shutdown on SIGTERM/SIGINT via `tokio::signal`.

## 6. SvelteKit Frontend

- [x] 6.1 Scaffold a SvelteKit project in `ui/` with `@sveltejs/adapter-static`, TypeScript, and a root layout displaying "flowl" with a navigation placeholder.
- [x] 6.2 Create `build.rs` that runs `npm install && npm run build` in `ui/` during `cargo build`.

## 7. Application Bootstrap

- [x] 7.1 Rewrite `src/main.rs` to wire everything together: parse config → init tracing → create DB pool → run migrations → start MQTT → start HTTP server.

## 8. Tests

- [x] 8.1 Add unit tests for config parsing (defaults, custom values, invalid values).
- [x] 8.2 Add an integration test for `GET /health` returning 200 with `{"status": "ok"}`.

## 9. Verify

- [x] 9.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` to confirm everything passes.
