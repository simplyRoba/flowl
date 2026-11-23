# Tasks

## 1. Configure the MQTT link origin

- [x] 1.1 Reuse `FLOWL_EXTERNAL_URL` for MQTT at startup. With OIDC enabled, use its validated origin; otherwise, validate the optional origin for enabled MQTT. Test LAN HTTP, HTTPS with and without a trailing slash, absent and invalid input, and unchanged OIDC requirements. Verify invalid values are not echoed.
- [x] 1.2 Update the existing `FLOWL_EXTERNAL_URL` entry in `README.md`. Verify the row describes the browser address and no-subpath constraint only. Keep MQTT link setup in the Home Assistant section and HTTPS requirements for OIDC in the OIDC section.

## 2. Publish and consume plant links

- [x] 2.1 Add optional `plant_url` to the shared attributes builder in `src/mqtt.rs`. Verify payload tests cover distinct plant IDs, a never-watered plant, a custom topic prefix, and omission of the key without an origin.
- [x] 2.2 Wire the validated origin into `AppState` and the state checker, then carry it through all attributes publish paths. Extend the fake-broker test in `tests/care_events.rs` to verify the initial create message and subsequent care-event updates carry the same URL. Add MQTT coverage for the initial/reconnect full sync and periodic reconciliation, and verify repair and post-restore repair use the same full-republish path.
- [x] 2.3 Add a Home Assistant dashboard or automation link example to `README.md`. Verify it reads `plant_url` from the sensor, opens `/plants/{id}` for that plant, and skips the link when the attribute is absent.

## 3. Verify the change

- [x] 3.1 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`.
