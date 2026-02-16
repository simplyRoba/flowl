## 1. Configuration and startup gating

- [x] 1.1 Surface `FLOWL_MQTT_DISABLED` in `Config::from_env`, defaulting to `false`, and expose a typed flag in the config struct.
- [x] 1.2 Pass the new flag through to `main.rs`, ensuring the logic that builds `mqtt::connect`, `spawn_state_checker`, and `AppState` can skip MQTT when the flag is true.

## 2. MQTT client & background tasks

- [x] 2.1 Update `mqtt::connect`, `mqtt::spawn_state_checker`, and any related helpers so they short-circuit when MQTT is disabled, logging that MQTT is disabled instead of retrying.
- [x] 2.2 Ensure `AppState` stores `None` for the client when MQTT is disabled and that API handlers/background code already tolerate the absence.

## 3. Documentation, testing, and verification

- [x] 3.1 Document the new flag in `README.md` (top level configuration section) also document all other environment currently available in the project variables in a new section for easy reference. do this as a markdown table.
- [x] 3.2 Add or update tests to cover disabling MQTT (e.g., config parsing, startup behavior, and API endpoints still working without an MQTT client).
- [x] 3.3 Run `cargo fmt`, `cargo clippy`, and `cargo test` after implementation to verify the change before closing the change.
