## Tasks

- [x] **Task 1: Add last_watered migration** — Create `migrations/20260215200000_add_last_watered.sql` adding `last_watered TEXT` column (nullable) to the `plants` table. Validates: `data/plants` — Plant Database Schema (last_watered column).

- [x] **Task 2: Add watering status computation to Plant response** — Add `watering_status` (String: `ok`/`due`/`overdue`), `last_watered` (Option<String>), and `next_due` (Option<String>) to the `Plant` response struct. Add `last_watered` to `PlantRow`. Implement status computation in `From<PlantRow> for Plant`: NULL last_watered → `due`, else compare today's date against `last_watered + watering_interval_days`. Update `PLANT_SELECT` to include `p.last_watered`. Add `chrono` dependency to `Cargo.toml` if not present. Validates: `data/plants` — Plant API Response — Watering Fields.

- [x] **Task 3: Add water plant endpoint** — Create `water_plant` handler in `src/api/plants.rs`: `POST /api/plants/:id/water` sets `last_watered = datetime('now')` and `updated_at = datetime('now')`, returns updated plant. Mount route in `src/api/mod.rs`. Validates: `data/plants` — Water Plant.

- [x] **Task 4: Add MQTT client to AppState** — Add `mqtt_client: Option<rumqttc::AsyncClient>` to `AppState` in `src/state.rs`. Add `mqtt_prefix: String` to `AppState` for topic prefix. Update `main.rs` to pass the MQTT `AsyncClient` and topic prefix into `AppState`. Keep the event loop `JoinHandle` in `main.rs`. Validates: `core/mqtt` — MQTT Client Shared via AppState.

- [x] **Task 5: Add MQTT publishing functions** — Add functions to `src/mqtt.rs`: `publish_discovery(client, prefix, plant_id, plant_name)` publishes retained HA auto-discovery JSON (including `json_attributes_topic`) to `homeassistant/sensor/{prefix}_plant_{id}/config`. `publish_state(client, prefix, plant_id, status)` publishes retained status string to `{prefix}/plant/{id}/state`. `publish_attributes(client, prefix, plant_id, last_watered, next_due, interval_days)` publishes retained JSON attributes to `{prefix}/plant/{id}/attributes`. `remove_plant(client, prefix, plant_id)` publishes empty retained payloads to discovery, state, and attributes topics. All functions take `Option<&AsyncClient>` and no-op when `None`. Log errors but never return `Err`. Validates: `core/mqtt` — HA Auto-Discovery, State Publishing, Attributes Publishing.

- [x] **Task 6: Trigger MQTT publishes from API handlers** — Update `create_plant` to call `publish_discovery` + `publish_state` + `publish_attributes` after creation. Update `update_plant` to call `publish_discovery` + `publish_state` + `publish_attributes` after update. Update `delete_plant` to call `remove_plant` after deletion. Update `water_plant` to call `publish_state` + `publish_attributes` after watering. Extract MQTT client and prefix from `AppState` in handlers that currently extract `State<SqlitePool>` (change to `State<AppState>` where needed). Validates: `data/plants` — MQTT Publishing on Plant State Changes.

- [x] **Task 7: Add background state checker** — Add `spawn_state_checker(pool, mqtt_client, prefix)` in `src/mqtt.rs` that spawns a tokio task running every 60 seconds. On first run and each tick: query all plants, compute status, compare against in-memory `HashMap<i64, String>` cache, publish state and attributes for changed plants. On first run also publish discovery configs for all plants. Call from `main.rs` after building `AppState`. Validates: `core/mqtt` — Background State Checker.

- [x] **Task 8: Write backend tests** — Add tests in `tests/api_plants.rs` for: water plant returns 200 with updated `last_watered` and `watering_status`; water plant 404 for missing plant; plant response includes `watering_status`/`last_watered`/`next_due` fields; newly created plant has `watering_status` = `due`; watering status computation (ok/due/overdue based on dates). Add unit tests for status computation logic. Validates: all `data/plants` specs.

- [x] **Task 9: Update frontend API client** — Add `watering_status`, `last_watered`, `next_due` to the `Plant` TypeScript interface in `ui/src/lib/api.ts`. Add `waterPlant(id: number): Promise<Plant>` function that sends `POST /api/plants/${id}/water`. Validates: `ui/plants` — API Client — Water Plant.

- [x] **Task 10: Update plant store** — Add `waterPlant(id: number)` to `ui/src/lib/stores/plants.ts` that calls `api.waterPlant(id)`, updates the plant list and `currentPlant` store. Validates: `ui/plants` — Plant Store — Water Plant.

- [x] **Task 11: Add watering indicators to dashboard** — Update `ui/src/routes/+page.svelte` to show a status badge on plant cards when `watering_status` is `due` (amber) or `overdue` (red). No indicator for `ok`. Validates: `ui/plants` — Dashboard Watering Status Indicators.

- [x] **Task 12: Add watering section to plant detail** — Update `ui/src/routes/plants/[id]/+page.svelte`: replace the static watering info card with a richer section showing watering status (colored indicator), last watered date (or "Never"), next due date, interval, and a "Water now" button. Wire button to call `waterPlant` from store. Validates: `ui/plants` — Plant Detail Watering Section.

- [x] **Task 13: Verify build and run** — Run `cargo fmt`, `cargo clippy` (no warnings), `cargo test` (all pass), build full app (`npm run build` + `cargo build`). Validates: end-to-end verification.
