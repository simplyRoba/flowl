## Why

Running Flowl without an MQTT broker (or while developing locally without the broker dependency) currently fails during startup and keeps the background checker alive even though nothing can be published. Adding a toggle to disable MQTT lets the service stay lightweight and start cleanly when MQTT integration is not needed.

## What Changes

- Introduce a `FLOWL_MQTT_DISABLED` configuration flag (default `false`) and teach `Config::from_env`/startup logic to honor it so the MQTT client is optional.
- Update the MQTT client lifecycle, background checker, and AppState plumbing so they are skipped when MQTT is disabled, ensuring HTTP APIs remain functional and no MQTT work runs.
- Document the new flag in the README/general config guidance so operators know how to run without MQTT.

## Capabilities

### New Capabilities
- _None_

### Modified Capabilities
- `core/mqtt`: Allow MQTT requirements to be fulfilled only when `FLOWL_MQTT_DISABLED` is false, describing how the client, background checker, and publishing behave when MQTT is opt-in.

## Impact

- `src/config.rs`, `src/main.rs`, and the MQTT-related modules to surface and respect the new flag.
- Background state checker logic that drives MQTT publications and the AppState field that shares the client.
- Documentation/README so operators know how to run the service without MQTT.
