## Why

Phase 2 established plant CRUD, photos, and the settings page, but plants have no watering lifecycle. The `watering_interval_days` field exists on every plant but nothing tracks when a plant was last watered, whether it's due, or notifies the user. This change adds the watering lifecycle: tracking last-watered timestamps, computing due/overdue status, exposing a "Water now" action, publishing plant state to MQTT for Home Assistant auto-discovery, and showing due/overdue indicators on the dashboard.

## What Changes

- Add a `last_watered` column to the `plants` table to record when each plant was last watered.
- Compute `watering_status` (`ok` / `due` / `overdue`) and `next_due` dynamically in the plant API response based on `last_watered` + `watering_interval_days`.
- Add a `POST /api/plants/:id/water` endpoint that sets `last_watered` to now and publishes updated state to MQTT.
- Share the MQTT `AsyncClient` via `AppState` so API handlers can publish messages.
- Publish Home Assistant MQTT auto-discovery configs for each plant, registering them as HA sensor entities.
- Publish watering state (`ok` / `due` / `overdue`) to retained MQTT state topics whenever state changes (water action, plant create/update/delete).
- Spawn a background task that periodically checks all plants and publishes state transitions (e.g., `ok` → `due` → `overdue`) to MQTT.
- Add due/overdue status indicators on dashboard plant cards.
- Add a "Water now" button on the plant detail view with watering status display.

## Capabilities

### Modified Capabilities

- `data/plants`: Add `last_watered` column, computed `watering_status` and `next_due` in response, "water now" endpoint.
- `core/mqtt`: Add HA auto-discovery publishing, state topic publishing, background state checker, MQTT client shared via `AppState`.
- `core/server`: Add MQTT client to `AppState`.
- `ui/plants`: Due/overdue indicators on dashboard cards, "Water now" button and watering status on detail view.

## Impact

- `migrations/`: New migration adding `last_watered` column to `plants`.
- `src/state.rs`: Add `MqttClient` (optional `AsyncClient`) to `AppState`.
- `src/mqtt.rs`: Add functions for publishing discovery configs, state messages, and removing entities. Add background state checker task.
- `src/api/plants.rs`: Add `watering_status`, `last_watered`, `next_due` to `Plant` response. Add `water_plant` handler. Trigger MQTT publishes on create/update/delete/water.
- `src/api/mod.rs`: Mount `/plants/:id/water` route.
- `src/main.rs`: Pass MQTT client to `AppState`, spawn background checker.
- `ui/src/lib/api.ts`: Add `waterPlant` function, update `Plant` type with new fields.
- `ui/src/lib/stores/plants.ts`: Add `waterPlant` store function.
- `ui/src/routes/+page.svelte`: Due/overdue indicators on plant cards.
- `ui/src/routes/plants/[id]/+page.svelte`: "Water now" button, watering status display.
