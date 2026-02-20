## Context

The MQTT integration currently operates as a fire-and-forget background subsystem. The event loop in `mqtt::connect()` logs `ConnAck` on connection and warnings on errors, but no runtime state is exposed to the rest of the application. The settings page has no MQTT section. Users must inspect server logs or environment variables to understand MQTT status.

The mockup defines an MQTT settings section showing: a status indicator with colored dot (Connected/Disabled), broker address, and topic prefix — with broker and prefix hidden when MQTT is disabled.

## Goals / Non-Goals

**Goals:**
- Track MQTT connection state at runtime so the API can report it
- Expose MQTT configuration and connection status via a single API endpoint
- Display MQTT status in the settings UI matching the mockup design

**Non-Goals:**
- Configuring MQTT from the UI (remains env-var driven)
- Showing MQTT message throughput or detailed diagnostics
- WebSocket-based live status updates (polling on page load is sufficient)

## Decisions

### 1. Connection tracking via `Arc<AtomicBool>`

Use `Arc<AtomicBool>` shared between the MQTT event loop and `AppState`. Set to `true` on `ConnAck`, set to `false` on connection error.

**Why over alternatives:**
- `Arc<Mutex<bool>>` — unnecessary locking overhead for a single boolean
- `watch` channel — more complex, no subscribers need notification push; the API polls on request
- Status enum (`Connected`/`Connecting`/`Disconnected`) — `AtomicBool` is simpler and sufficient; the UI only needs connected vs not-connected

### 2. Store MQTT config in `AppState`

Add `mqtt_host`, `mqtt_port`, `mqtt_prefix`, and `mqtt_disabled` fields to `AppState` so the status endpoint can return them without re-reading environment variables.

**Why:** Config is already parsed once at startup. Passing it through `AppState` follows the existing pattern (pool, upload_dir are already there). Avoids coupling the API handler to `Config` directly.

### 3. Single `GET /api/mqtt/status` endpoint

Return a JSON object with `status` ("connected" | "disconnected" | "disabled"), `broker` ("host:port" or null), and `topic_prefix` (string or null). Broker and prefix are null when disabled.

**Why over extending `/api/info`:** The info endpoint is static compile-time metadata. MQTT status is runtime state — mixing them blurs the boundary. A separate endpoint is also independently cacheable and testable.

### 4. UI section hidden when API fails, rows hidden when disabled

Follow the same pattern as the About and Data sections: fetch on mount, hide the section entirely if the API call fails. When MQTT is disabled, show only the status row with "Disabled" text — no broker or prefix rows.

## Risks / Trade-offs

- **AtomicBool race with event loop** → Acceptable. The status is advisory; a brief stale value between `ConnAck` and the next API request is harmless.
- **No "connecting" state** → The initial state is `false` (disconnected). Between startup and first `ConnAck`, the UI shows "Disconnected". This is accurate — the client isn't connected yet. Keeping the model binary avoids a third state that would complicate both backend and frontend.
- **MQTT config fields widen AppState** → Four small fields (`String`, `u16`, `String`, `bool`). The struct is cloned once into the router; the overhead is negligible.
