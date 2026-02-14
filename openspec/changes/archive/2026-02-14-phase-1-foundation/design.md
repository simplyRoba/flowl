## Context

The project is a bare Rust crate (`cargo init`) with CI/CD workflows, a Dockerfile, and no application code. Phase 1 establishes the runtime foundation: HTTP server, database, frontend embedding, and MQTT connectivity. All subsequent features (plant CRUD, watering schedules, care log) build on this skeleton.

## Goals / Non-Goals

**Goals:**
- A running Axum server that responds to `/health` and serves the SvelteKit SPA
- SQLite database with sqlx migration infrastructure, auto-created on startup
- MQTT client that connects to a Mosquitto broker and stays connected
- Environment-based configuration with sensible defaults
- Structured logging via `tracing`
- Single binary with embedded frontend assets

**Non-Goals:**
- No plant CRUD, watering logic, or care log (phase 2+)
- No MQTT publishing logic (phase 3) — only the connection lifecycle
- No authentication or authorization (single-user, local network)
- No frontend routes or components beyond the empty SvelteKit shell

## Decisions

### Decision 1: Axum with Tower middleware

Use Axum as the HTTP framework. It integrates natively with Tower and Tokio, has strong ecosystem support, and keeps the dependency tree small. Use `tower-http` for static file serving and compression.

**Alternative considered:** Actix-web — heavier, uses its own runtime, unnecessary for this scope.

### Decision 2: sqlx with runtime migrations

Use sqlx in offline mode for compile-time query checking where possible. Run migrations at startup via `sqlx::migrate!()` so the database schema is always up to date when the binary starts. Store the SQLite file at a configurable path (`FLOWL_DB_PATH`, default `/data/flowl.db`) so Docker volume mounts work naturally.

**Alternative considered:** rusqlite — synchronous, no compile-time query checking, poorer Tokio integration.

### Decision 3: rust-embed for SvelteKit assets

Embed the SvelteKit build output (`ui/build/`) into the binary at compile time using `rust-embed`. Serve these assets via a fallback route so the SPA handles client-side routing. A `build.rs` script runs `npm run build` in the `ui/` directory before compilation.

**Alternative considered:** Serve from filesystem at runtime — adds a deployment dependency, breaks the single-binary goal.

### Decision 4: rumqttc for MQTT

Use `rumqttc` as the async MQTT client. It runs on Tokio, supports MQTTv5, and handles reconnection internally. The client connects on startup in a background task and logs connection state. No publishing or subscribing happens in phase 1 — the connection lifecycle is the only concern.

**Alternative considered:** paho-mqtt — C bindings, larger binary, harder cross-compilation.

### Decision 5: Environment-based configuration

Parse all configuration from environment variables at startup using a simple config struct with `std::env`. No config file, no CLI args. This keeps the Docker setup clean (`-e` flags or `.env` file).

| Variable                  | Default          |
|---------------------------|------------------|
| `FLOWL_PORT`              | `8080`           |
| `FLOWL_DB_PATH`           | `/data/flowl.db` |
| `FLOWL_MQTT_HOST`         | `localhost`      |
| `FLOWL_MQTT_PORT`         | `1883`           |
| `FLOWL_MQTT_TOPIC_PREFIX` | `flowl`          |
| `FLOWL_LOG_LEVEL`         | `info`           |

**Alternative considered:** `clap` + config file — over-engineered for a single-user self-hosted service.

### Decision 6: Module structure

Organize the crate into focused modules:

```
src/
├── main.rs          # Bootstrap: config → DB → MQTT → server
├── config.rs        # Environment parsing, Config struct
├── db.rs            # SQLite pool creation, migration runner
├── mqtt.rs          # MQTT client lifecycle (connect, background task)
├── server.rs        # Axum router, routes, static file serving
└── embedded.rs      # rust-embed asset struct and handler
```

Keep it flat — no nested module trees until complexity demands it.

### Decision 7: SvelteKit with adapter-static

Use SvelteKit with `@sveltejs/adapter-static` to produce a fully static build. The output is plain HTML/CSS/JS that can be embedded without a Node runtime. SvelteKit's file-based routing still works for client-side navigation.

```
ui/
├── package.json
├── svelte.config.js
├── vite.config.ts
├── src/
│   ├── app.html
│   ├── routes/
│   │   └── +layout.svelte
│   └── lib/
└── static/
```

## Risks / Trade-offs

- **Build complexity**: `build.rs` running `npm run build` requires Node.js in the build environment. → Mitigated by the devcontainer which includes Node LTS. CI also needs Node installed before `cargo build`.
- **sqlx offline mode**: Compile-time query checking needs a `sqlx-data.json` or `DATABASE_URL` at build time. → Use `sqlx prepare` to generate offline data, or skip compile-time checks initially and use runtime-checked queries.
- **MQTT reconnection**: If the broker is unavailable at startup, the service should still start and serve HTTP. → rumqttc reconnects automatically; log warnings but don't block startup.
- **Large binary**: Embedding frontend assets increases binary size. → Acceptable trade-off for single-binary deployment; SvelteKit builds are small.
