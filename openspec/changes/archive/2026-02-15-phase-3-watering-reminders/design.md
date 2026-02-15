## Context

Phase 2 gave each plant a `watering_interval_days` field, but nothing tracks actual watering events or computes due status. The MQTT client connects to the broker but does not publish any topics. Phase 3 closes this gap: plants gain a watering lifecycle with state tracking, the MQTT client publishes HA auto-discovery configs and watering state, and the UI surfaces due/overdue indicators with a "Water now" action.

## Goals / Non-Goals

**Goals:**
- Track when each plant was last watered via a `last_watered` timestamp
- Compute watering status (`ok` / `due` / `overdue`) and `next_due` date dynamically in API responses
- "Water now" endpoint that records a watering event and publishes state to MQTT
- MQTT auto-discovery: register each plant as a Home Assistant sensor entity
- Publish watering state to retained MQTT topics on every state-changing action
- Background task that detects and publishes state transitions periodically
- Dashboard cards show due/overdue visual indicators
- Plant detail view shows watering status and "Water now" button

**Non-Goals:**
- No care log entries for watering events (phase 4 — care journal)
- No multiple watering schedules per plant (single interval is sufficient)
- No push notifications from flowl itself (HA automations handle notifications)
- No seasonal adjustments to watering intervals (phase 5)
- No watering history or statistics view

## Decisions

### Decision 1: last_watered as nullable datetime on plants table

Add `last_watered TEXT` (nullable, ISO 8601) directly to the `plants` table rather than creating a separate watering schedule table. The plan's initial data model envisioned a `watering_schedule` table, but since `watering_interval_days` already lives on `plants`, keeping `last_watered` there too is simpler and avoids a join for every plant query. When `last_watered` is NULL, the plant has never been watered.

**Alternative considered:** Separate `watering_schedules` table — adds join complexity for no benefit in a single-schedule-per-plant model.

### Decision 2: Watering status computed server-side, not stored

Compute `watering_status` and `next_due` in the API response layer rather than storing them in the database. This avoids stale data: status changes over time without any write occurring (an `ok` plant becomes `due` simply because time passes). The computation is cheap (date arithmetic) and happens in the Rust response mapping.

Status logic:
- If `last_watered` is NULL → `due` (never watered, needs attention)
- Else compute `next_due = last_watered + watering_interval_days`:
  - If today > next_due → `overdue`
  - If today = next_due → `due`
  - If today < next_due → `ok`

Date comparisons use date-only granularity (ignore time component) so a plant watered at 23:00 isn't considered due the next morning.

### Decision 3: Water now endpoint

`POST /api/plants/:id/water` sets `last_watered = datetime('now')` and `updated_at = datetime('now')`, returns the updated plant with recomputed status. No request body needed. This endpoint also triggers MQTT state publishing.

**Alternative considered:** Reusing `PUT /api/plants/:id` with `{"last_watered": "now"}` — conflates data editing with a user action; a dedicated endpoint is clearer and easier to protect/audit.

### Decision 4: MQTT client optional in AppState

Wrap the MQTT `AsyncClient` in `Option<AsyncClient>` inside `AppState`. If MQTT connection fails or is not configured, API handlers still work — they skip publishing. This makes MQTT a best-effort enhancement rather than a hard dependency.

The `MqttHandle` struct is split: `AsyncClient` goes into `AppState`, the event loop `JoinHandle` stays in `main.rs` for lifecycle management.

### Decision 5: Home Assistant MQTT auto-discovery format

For each plant, publish a retained JSON config to `homeassistant/sensor/flowl_plant_{id}/config`:

```json
{
  "name": "{plant_name}",
  "unique_id": "flowl_plant_{id}",
  "state_topic": "{prefix}/plant/{id}/state",
  "json_attributes_topic": "{prefix}/plant/{id}/attributes",
  "icon": "mdi:flower",
  "device": {
    "identifiers": ["flowl"],
    "name": "flowl",
    "manufacturer": "flowl"
  }
}
```

State is published to `{prefix}/plant/{id}/state` as a simple string: `ok`, `due`, or `overdue`.

Attributes are published to `{prefix}/plant/{id}/attributes` as a JSON object:

```json
{
  "next_due": "2026-02-20",
  "last_watered": "2026-02-13T14:30:00",
  "watering_interval_days": 7
}
```

This lets HA users display dates on dashboards and use them in automation templates. All three topics (config, state, attributes) use retained messages so HA picks them up on restart.

To remove a plant from HA, publish empty payloads to its discovery, state, and attributes topics (HA convention for entity removal).

### Decision 6: MQTT publishing triggers

Publish MQTT messages on these events:
- **Plant created**: Publish discovery config + initial state + attributes
- **Plant updated**: Re-publish discovery config (name may have changed) + state + attributes
- **Plant watered**: Publish state + attributes
- **Plant deleted**: Publish empty payloads to discovery, state, and attributes topics (removes from HA)
- **Background check**: Publish state for any plant whose status has changed since last check

API handlers call MQTT publish functions but do not await delivery confirmation — fire and forget. MQTT errors are logged but never fail the API response.

### Decision 7: Background state checker

Spawn a tokio task in `main.rs` that runs every 60 seconds. On each tick it:
1. Queries all plants with their `last_watered` and `watering_interval_days`
2. Computes current status for each
3. Compares against last-published status (kept in memory)
4. Publishes state updates for any plants that have transitioned

This ensures HA reflects time-based transitions (ok → due → overdue) even when no user interaction occurs. The in-memory cache is a simple `HashMap<i64, String>` of plant_id → last published status.

### Decision 8: Dashboard due/overdue indicators

Add a small colored badge/dot on dashboard plant cards:
- `overdue`: red indicator with "Overdue" label
- `due`: amber/orange indicator with "Due" label
- `ok`: no indicator (clean default state)

This provides at-a-glance watering status without cluttering the card design.

### Decision 9: Plant detail watering section

Replace the static "Every N days" watering info card with a richer section showing:
- Current watering status with colored indicator
- Last watered date (or "Never" if null)
- Next due date
- "Water now" button that calls the water endpoint and refreshes the view

## Risks / Trade-offs

- **No persistent MQTT state cache**: The background checker's in-memory status cache resets on restart, causing a full re-publish of all plant states on startup. This is acceptable — HA handles duplicate retained messages gracefully, and the plant count is small.
- **No watering history**: "Water now" overwrites `last_watered` with no historical record. Phase 4's care journal will add a persistent log.
- **Date-only status granularity**: Status uses date comparison, so a plant watered at 23:59 won't be due again until `interval_days` later by calendar date. This matches user expectations for daily plant care.
- **MQTT best-effort**: If the broker is down, state updates are lost until the next background check after reconnection. HA users may see stale state briefly. Acceptable for a plant care app.
