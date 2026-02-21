## 1. Core Repair Logic

- [x] 1.1 Add `republish_all` function in `src/mqtt.rs` that queries all plants from SQLite and publishes discovery, state, and attributes for each
- [x] 1.2 Add `discover_broker_plant_ids` function that creates a temporary MQTT client, subscribes to the three wildcard topic patterns, collects retained messages with a silence timeout, extracts plant IDs from topic names, and returns the set of discovered IDs
- [x] 1.3 Add `repair` function that calls `discover_broker_plant_ids`, diffs against current DB plant IDs, clears all three topics for each orphan via `remove_plant`, then calls `republish_all`, and returns the cleared/published counts
- [x] 1.4 Write unit tests for plant ID extraction from topic names (parsing `homeassistant/sensor/{prefix}_plant_{id}/config`, `{prefix}/plant/{id}/state`, `{prefix}/plant/{id}/attributes`)

## 2. Reconnect-Triggered Republish

- [x] 2.1 Update `spawn_state_checker` signature to accept `Arc<AtomicBool>` for connection status
- [x] 2.2 Add reconnect detection logic: track previous connection state each tick, trigger `republish_all` on `false → true` transition
- [x] 2.3 Update `main.rs` to pass the `Arc<AtomicBool>` to `spawn_state_checker`
- [x] 2.4 Write unit test for `spawn_state_checker` confirming it accepts the new parameter when MQTT is disabled (returns `None`)

## 3. API Endpoint

- [x] 3.1 Create `src/api/mqtt_repair.rs` with `post_mqtt_repair` handler that guards on MQTT disabled (409) and disconnected (503), then calls `mqtt::repair`
- [x] 3.2 Register `POST /mqtt/repair` route in `src/api/mod.rs`
- [x] 3.3 Write integration tests: repair returns 409 when MQTT disabled, repair returns expected JSON shape when MQTT enabled (using `mqtt_disabled: true` test setup)

## 4. Frontend

- [x] 4.1 Add `repairMqtt` API function in `ui/src/lib/api.ts` that calls `POST /api/mqtt/repair` and returns the `{ cleared, published }` response
- [x] 4.2 Add "Repair" button to the MQTT section in `ui/src/routes/settings/+page.svelte` — visible when MQTT is not disabled, enabled only when connected
- [x] 4.3 Add loading state, inline success message (cleared/published counts), and inline error handling for the repair button

## 5. Verification

- [x] 5.1 Run `cd ui && npm run check` to verify Svelte/TypeScript compilation
- [x] 5.2 Run `cargo fmt`, `cargo clippy`, and `cargo test` to verify Rust code quality and all tests pass
- [x] 5.3 Update README.md if the repair feature warrants user-facing documentation
