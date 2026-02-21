## 1. Migration

- [x] 1.1 Create migration that backfills a `watered` care event for any plant with `last_watered` set but no corresponding `watered` care event, then drops the `last_watered` column from `plants`

## 2. Backend — plant queries and water handler

- [x] 2.1 Update `PLANT_SELECT` to compute `last_watered` via subquery `(SELECT MAX(occurred_at) FROM care_events WHERE plant_id = p.id AND event_type = 'watered')` and remove `last_watered` from `PlantRow`
- [x] 2.2 Update MQTT `CheckerRow` query and `republish_all` query to use the same subquery instead of `plants.last_watered`
- [x] 2.3 Simplify `water_plant` handler: remove `UPDATE plants SET last_watered`, insert care event as required (not fire-and-forget), re-read plant, publish MQTT
- [x] 2.4 Update backup export query to compute `last_watered` via subquery; update restore to skip `last_watered` from import JSON
- [x] 2.5 Update backend tests for `water_plant`, plant queries, and backup/restore to reflect computed `last_watered`

## 3. Backend — care event MQTT publishing

- [x] 3.1 Switch `create_care_event` and `delete_care_event` handlers from `State<SqlitePool>` to `State<AppState>`
- [x] 3.2 In `create_care_event`, publish MQTT state and attributes after inserting a `watered` event
- [x] 3.3 In `delete_care_event`, read event type before deleting; publish MQTT state and attributes after deleting a `watered` event
- [x] 3.4 Add backend tests for MQTT publishing on watered care event create and delete

## 4. Frontend — reload plant after care event changes

- [x] 4.1 Update `removeCareEvent` in care store or `handleEventDelete` on the detail page to reload the plant after deletion
- [x] 4.2 Update frontend tests for care event deletion to verify plant data is reloaded

## 5. Checks

- [x] 5.1 Run `ui/npm run check`, `cargo fmt`, `cargo clippy`, and `cargo test`
