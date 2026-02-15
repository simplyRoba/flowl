## 1. Database

- [x] 1.1 Create `migrations/20260215300000_add_care_events.sql` adding the `care_events` table with `id`, `plant_id` (FK to plants ON DELETE CASCADE), `event_type`, `notes`, `occurred_at`, and `created_at` columns.

## 2. Backend API

- [x] 2.1 Create `src/api/care_events.rs` with `CareEvent` response struct and `CareEventRow` query struct. Add `list_care_events` handler (`GET /api/plants/:id/care`) that verifies the plant exists and returns care events ordered by `occurred_at` DESC.
- [x] 2.2 Add `create_care_event` handler (`POST /api/plants/:id/care`) that validates `event_type` against the allowed set (`watered`, `fertilized`, `repotted`, `pruned`, `custom`), verifies the plant exists, defaults `occurred_at` to `datetime('now')` when omitted, and returns the created event with HTTP 201.
- [x] 2.3 Add `delete_care_event` handler (`DELETE /api/plants/:id/care/:event_id`) that verifies the event belongs to the plant and returns HTTP 204 on success or 404 if not found.
- [x] 2.4 Add `list_all_care_events` handler (`GET /api/care`) with cursor-based pagination (`?limit=N&before=ID`) and optional `?type=` filter. Join with plants table to include `plant_name`. Return `{ events: [...], has_more: bool }`. Default limit 20, max 100. Return 422 for invalid type filter.
- [x] 2.5 Mount care event routes in `src/api/mod.rs` under `/plants/{id}/care`, `/plants/{id}/care/{event_id}`, and `/care`.
- [x] 2.6 Update `water_plant` handler in `src/api/plants.rs` to insert a care event with `event_type = "watered"` after updating `last_watered`.

## 3. Backend Tests

- [x] 3.1 Add integration tests in `tests/api_care_events.rs`: list empty, create valid event, create with explicit `occurred_at`, create with invalid type (422), create with missing type (422), create for nonexistent plant (404), delete event, delete nonexistent event (404), list returns events ordered by `occurred_at` DESC, response includes `plant_name`.
- [x] 3.2 Add integration tests for `GET /api/care`: returns events across plants, respects `limit` parameter, cursor-based pagination with `before` parameter, `has_more` flag correct, `type` filter works, invalid type returns 422, empty when no events.
- [x] 3.3 Add test that `POST /api/plants/:id/water` auto-creates a `watered` care event (verify via `GET /api/plants/:id/care`).
- [x] 3.4 Add test that deleting a plant cascades to its care events.

## 4. Frontend API & Store

- [x] 4.1 Add `CareEvent`, `CreateCareEvent`, and `CareEventsPage` interfaces to `ui/src/lib/api.ts`. Add `fetchCareEvents(plantId)`, `fetchAllCareEvents(limit?, before?, type?)`, `createCareEvent(plantId, data)`, and `deleteCareEvent(plantId, eventId)` functions.
- [x] 4.2 Create `ui/src/lib/stores/care.ts` with `careEvents` writable store, `loadCareEvents(plantId)`, `addCareEvent(plantId, data)`, and `removeCareEvent(plantId, eventId)` functions.

## 5. Frontend UI

- [x] 5.1 Add care journal timeline section to `ui/src/routes/plants/[id]/+page.svelte` below the watering card. Load care events on mount. Display events grouped by day ("Today", "Yesterday", or date) with type icon, label, date, and notes. Show empty state when no events. Limit initial display to 20 events with a "Show more" link.
- [x] 5.2 Add "+ Add log entry" link below the care journal timeline. When clicked, show an inline form with event type chips (Fertilized, Repotted, Pruned, Custom), optional notes field, and Save/Cancel buttons. On save, call `addCareEvent` and refresh the timeline.
- [x] 5.3 Add a delete button on each care event in the timeline. On click, call `removeCareEvent` and remove the event from the list.
- [x] 5.4 Create `ui/src/routes/log/+page.svelte` as the global care log page. Fetch events from `GET /api/care`. Display events grouped by day with plant name (linked to detail view), event type icon, time, and notes. Add filter chips (All, Watered, Fertilized, Repotted, Pruned, Custom) that pass `?type=` to the API. Implement infinite scrolling that automatically fetches the next page when the user scrolls near the bottom. Show empty state when no events exist.

## 6. Verification

- [x] 6.1 Run `cargo fmt`, `cargo clippy` (no warnings), `cargo test` (all pass), and build full app (`npm run build` + `cargo build`).
