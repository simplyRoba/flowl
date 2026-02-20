## Why

The settings page has no visibility into MQTT integration status. Users cannot tell whether MQTT is enabled, whether the connection to the broker is alive, or what broker/prefix is configured — they must check environment variables or server logs.

## What Changes

- Add runtime MQTT connection status tracking (`connected` / `disconnected` / `disabled`) using an `AtomicBool` updated in the MQTT event loop
- Expose MQTT status via a new `GET /api/mqtt/status` endpoint returning connection state, broker address, and topic prefix
- Add an "MQTT" section to the settings page showing status (with colored indicator dot), broker, and topic prefix
- When MQTT is disabled, show only the status row ("Disabled") — broker and prefix rows are hidden

## Capabilities

### New Capabilities

- `core/mqtt-status`: API endpoint exposing MQTT runtime connection state and configuration (broker, prefix, enabled/disabled)

### Modified Capabilities

- `core/mqtt`: ADDED connection status tracking via shared `AtomicBool` updated on `ConnAck` (connected) and connection error (disconnected)
- `ui/settings`: ADDED MQTT section displaying connection status, broker address, and topic prefix

## Impact

- `src/mqtt.rs`: Accept and update a shared `Arc<AtomicBool>` for connection state in `connect()`
- `src/state.rs`: Add `mqtt_connected: Arc<AtomicBool>` and MQTT config fields (host, port, prefix, disabled) to `AppState`
- `src/main.rs`: Wire the new shared state through to `AppState`
- `src/api/`: New handler for `GET /api/mqtt/status`
- `ui/src/lib/api.ts`: New `MqttStatus` type and `fetchMqttStatus()` function
- `ui/src/routes/settings/+page.svelte`: New MQTT section with status indicator
