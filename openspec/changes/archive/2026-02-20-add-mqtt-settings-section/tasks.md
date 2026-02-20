## 1. Backend: Connection status tracking

- [x] 1.1 Add `Arc<AtomicBool>` parameter to `mqtt::connect()`, set `true` on `ConnAck`, set `false` on connection error
- [x] 1.2 Extend `AppState` with `mqtt_connected: Option<Arc<AtomicBool>>`, `mqtt_host: String`, `mqtt_port: u16`, `mqtt_disabled: bool`
- [x] 1.3 Wire the new fields in `main.rs` — create the `AtomicBool`, pass to `connect()`, store in `AppState`

## 2. Backend: MQTT status endpoint

- [x] 2.1 Create `src/api/mqtt_status.rs` with `get_mqtt_status` handler returning `{ status, broker, topic_prefix }`
- [x] 2.2 Register the route as `GET /api/mqtt/status` in `src/api/mod.rs`
- [x] 2.3 Write integration tests: disabled returns `{ "status": "disabled", "broker": null, "topic_prefix": null }`; enabled-but-no-broker returns `disconnected` with broker and prefix populated

## 3. Frontend: API and settings UI

- [x] 3.1 Add `MqttStatus` type and `fetchMqttStatus()` function to `ui/src/lib/api.ts`
- [x] 3.2 Add MQTT section to `ui/src/routes/settings/+page.svelte` — fetch on mount, show status row with colored dot, broker row, topic prefix row; hide section on fetch failure; hide broker/prefix when disabled

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`
- [x] 4.2 Run `npx svelte-check` in the UI
