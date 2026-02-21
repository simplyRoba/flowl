## Context

`last_watered` is a stored column on the `plants` table, only updated by the `water_plant` handler via `UPDATE plants SET last_watered = datetime('now')`. That handler also inserts a `watered` care event. However, deleting a care event or manually logging one through the care journal API does not update `last_watered`, causing it to drift from the actual care history. The fields `watering_status` and `next_due` are already computed at read time from `last_watered` + `watering_interval_days` via `compute_watering_status()` in `plants.rs`.

The care event handlers currently extract `State<SqlitePool>` (not `State<AppState>`), so they have no access to the MQTT client. The `SqlitePool` is available via `FromRef<AppState>`.

Backup/restore exports `last_watered` as part of the plant JSON. The MQTT background checker reads `last_watered` directly from the `plants` table.

## Goals / Non-Goals

**Goals:**
- Make `last_watered` a computed value derived from `care_events` — single source of truth.
- Keep `last_watered` in the API response (same shape, same field name) — no frontend API changes.
- Publish MQTT state when watered care events are created or deleted.
- Update the frontend to reload the plant after deleting a care event.

**Non-Goals:**
- Changing the care event data model or API shape.
- Adding database triggers — keep logic in application code.
- Changing how `watering_status` / `next_due` are computed (that logic stays the same).

## Decisions

- Compute `last_watered` via a correlated subquery in `PLANT_SELECT` rather than a JOIN with GROUP BY.
  - **Alternative:** LEFT JOIN with `GROUP BY p.id`. **Rejected** — the existing query already JOINs locations, adding a GROUP BY would require listing all columns in the GROUP BY clause. A subquery is self-contained and doesn't affect the rest of the SELECT.
  - The subquery: `(SELECT MAX(occurred_at) FROM care_events WHERE plant_id = p.id AND event_type = 'watered') AS last_watered`.

- Drop the `last_watered` column via a SQLite migration. SQLite supports `ALTER TABLE DROP COLUMN` since 3.35.0 (2021). The Docker image uses a recent SQLite version.
  - **Alternative:** Keep the column but stop writing to it. **Rejected** — a dead column invites confusion and bugs.

- The `water_plant` handler becomes: insert care event → re-read plant (with computed `last_watered`) → publish MQTT. No more `UPDATE plants SET last_watered`. The care event insert is promoted from fire-and-forget to required (errors propagate).

- Switch care event handlers from `State<SqlitePool>` to `State<AppState>` for MQTT access. Only `create_care_event` and `delete_care_event` need MQTT — only when `event_type = 'watered'`. List endpoints stay on `SqlitePool`.

- The MQTT background checker (`CheckerRow`) switches its query to include the same subquery instead of reading `plants.last_watered`.

- Backup export: compute `last_watered` in the export query (same subquery). The export JSON shape doesn't change. Restore: ignore `last_watered` from import JSON — it's derived from the imported care events. No special handling needed since care events are restored alongside plants.

- Frontend: after `removeCareEvent`, reload the plant via `loadPlant` so `last_watered` / `watering_status` update in the UI. Same for `addCareEvent` when the type is `watered`.

## Risks / Trade-offs

- Subquery on every plant read adds a small cost. For this app's scale (dozens of plants, not thousands) it's negligible. SQLite can use the existing index on `care_events(plant_id)`.
  → If performance ever matters, a covering index on `(plant_id, event_type, occurred_at)` would help.

- Promoting the care event insert in `water_plant` from fire-and-forget to required means a DB error there now fails the request. This is the correct behavior — if the event can't be recorded, the watering didn't happen.

- The migration drops a column, which is irreversible. Data is preserved because every `water_plant` call already inserts a care event.
  → The migration backfills a care event for any plant with `last_watered` set but no corresponding `watered` care event, covering edge cases from before care events existed.
