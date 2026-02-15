## Why

Phase 3 added watering tracking but "Water now" overwrites `last_watered` with no historical record. Users have no way to see when they last cared for a plant, spot patterns, or verify past actions. A care journal adds a persistent event log for watering and other care activities (fertilizing, repotting, pruning), giving users a timeline of care per plant and enabling future features like statistics and streaks.

## What Changes

- Add a `care_events` table to store timestamped care entries per plant, with event type and optional notes.
- Add REST API endpoints for care events: list events for a plant, create an event, delete an event.
- Add a global `GET /api/care` endpoint returning paginated care events across all plants (with plant name included), powering the `/log` page.
- Automatically log a care event when the "Water now" action is used (type `watered`).
- Add a care journal section to the plant detail view showing a chronological timeline of care events.
- Add a manual "Log care" action on the plant detail view for recording non-watering care (fertilized, repotted, pruned, other).
- Update the frontend API client and plant store to support care event operations.
- Publish MQTT attributes update when care events change (updated `last_watered` reflected in attributes).

## Capabilities

### New Capabilities

- `data/care-events`: Care event database schema, CRUD API endpoints, and auto-logging on water action.
- `ui/care-journal`: Care journal timeline on plant detail view, manual care logging UI, global `/log` page with paginated care feed.

### Modified Capabilities

- `data/plants`: Water plant endpoint also creates a care event record.
- `ui/plants`: Plant detail view gains a care journal section.

## Impact

- `migrations/`: New migration creating `care_events` table.
- `src/api/`: New `care_events.rs` module with list/create/delete handlers.
- `src/api/mod.rs`: Mount care event routes under `/api/plants/:id/care` and `/api/care`.
- `src/api/plants.rs`: `water_plant` handler inserts a care event after updating `last_watered`.
- `ui/src/lib/api.ts`: Add care event types and API functions.
- `ui/src/lib/stores/`: New care events store or extend plant store.
- `ui/src/routes/plants/[id]/+page.svelte`: Add care journal timeline section and "Log care" button.
- `ui/src/routes/log/+page.svelte`: New global care log page with paginated event feed.
