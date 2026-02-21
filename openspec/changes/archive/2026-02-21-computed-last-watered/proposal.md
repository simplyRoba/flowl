## Why

`last_watered` is stored as a column on the `plants` table and only updated by the `water_plant` handler. Deleting a watered care event or manually logging one via the care journal does not update it, causing the watering status to drift out of sync with the actual care history. Making `last_watered` a computed value derived from `care_events` eliminates this class of bugs by design.

## What Changes

- Remove the `last_watered` column from the `plants` table via migration.
- Compute `last_watered` at query time as `MAX(occurred_at)` from `care_events` where `event_type = 'watered'` for each plant.
- Simplify the `water_plant` handler to only insert a care event (no more direct `UPDATE plants SET last_watered`).
- Add MQTT state publishing to `create_care_event` and `delete_care_event` when the event type is `watered`, since these actions now affect watering status.
- Reload the plant on the frontend after deleting a care event so the UI reflects updated `last_watered` / `watering_status`.
- Add a data migration to backfill a `watered` care event for any plant that has `last_watered` set but no corresponding care event.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `data/plants`: Remove `last_watered` from the stored schema. Compute it from `care_events` in plant queries. Simplify `water_plant` to only insert a care event.
- `data/care-events`: Add MQTT publishing on create/delete when `event_type = 'watered'`. Update delete handler to return the plant for frontend refresh.
- `ui/plants`: Reload plant data after deleting a care event so watering status updates in the UI.

## Impact

- Migration: drop `last_watered` column, backfill care events from existing data
- Modified: `src/api/plants.rs` (plant queries, `water_plant` handler)
- Modified: `src/api/care_events.rs` (MQTT publish on watered event create/delete)
- Modified: `ui/src/lib/stores/care.ts` (reload plant after delete)
- Modified: `ui/src/routes/plants/[id]/+page.svelte` (reload plant after event delete)
- Tests: update backend and frontend tests for new behavior
- No new dependencies
