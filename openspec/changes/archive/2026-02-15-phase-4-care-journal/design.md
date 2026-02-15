## Context

Phase 3 added watering lifecycle tracking: `last_watered` on the plants table, computed `watering_status`, a "Water now" endpoint, and MQTT state publishing. However, "Water now" overwrites `last_watered` with no history. Users cannot see when they previously watered, fertilized, or repotted a plant. Phase 4 introduces a `care_events` table to persistently log every care action, a REST API to manage events, and a timeline UI on the plant detail view.

The existing codebase follows a consistent pattern: SQLite + sqlx migrations, Axum handlers extracting `State<AppState>` or `State<SqlitePool>`, Svelte frontend with stores and a typed API client. This change adds a new entity (`care_events`) alongside the existing `plants` entity, following the same patterns.

## Goals / Non-Goals

**Goals:**
- Persist every care action as a timestamped event with type and optional notes
- Support care event types: `watered`, `fertilized`, `repotted`, `pruned`, `custom`
- Auto-log a `watered` event when the existing "Water now" endpoint is called
- REST API for listing, creating, and deleting care events per plant
- Global `GET /api/care` endpoint with cursor-based pagination for the `/log` page
- Timeline UI on the plant detail view showing recent care history
- Global `/log` page showing a paginated care feed across all plants
- Manual "Log care" action for non-watering events

**Non-Goals:**
- No care event editing (events are immutable records — delete and re-create if wrong)
- No care statistics, streaks, or analytics (future phase)
- No photo attachments on care events
- No search across care events (filtering by type is supported)
- No MQTT publishing of individual care events (watering already publishes state)

## Decisions

### Decision 1: care_events table schema

Create a `care_events` table with:
- `id` INTEGER PRIMARY KEY
- `plant_id` INTEGER NOT NULL, FK to `plants.id` ON DELETE CASCADE
- `event_type` TEXT NOT NULL (one of: `watered`, `fertilized`, `repotted`, `pruned`, `custom`)
- `notes` TEXT (nullable, free-form user notes)
- `occurred_at` TEXT NOT NULL (ISO 8601 datetime, when the event happened)
- `created_at` TEXT NOT NULL DEFAULT (datetime('now'))

`occurred_at` is separate from `created_at` to allow backdating events (e.g., "I watered this yesterday"). Auto-logged watering events use `datetime('now')` for both. ON DELETE CASCADE ensures events are cleaned up when a plant is deleted.

**Alternative considered:** Adding events to the plants table as JSON array — breaks normalization and makes querying expensive as history grows.

### Decision 2: Fixed event types as TEXT, not a separate table

Store `event_type` as a plain TEXT column validated at the application layer rather than creating a lookup table. The set of event types is small and stable. The `custom` type with a notes field covers edge cases. Application-layer validation returns 422 for unknown types.

**Alternative considered:** Enum table with FK — adds join complexity for no practical benefit with 5 fixed types.

### Decision 3: API routes

Mount care event CRUD routes under the plant resource:
- `GET /api/plants/:id/care` — list care events for a plant, ordered by `occurred_at` DESC
- `POST /api/plants/:id/care` — create a care event (body: `{ "event_type": "fertilized", "notes": "..." , "occurred_at": "..." }`)
- `DELETE /api/plants/:id/care/:event_id` — delete a single care event

This nesting reflects the ownership relationship. `occurred_at` in the POST body is optional; defaults to `datetime('now')` when omitted.

Additionally, a global read-only endpoint `GET /api/care?limit=N&before=ID` returns care events across all plants with cursor-based pagination, powering the `/log` page. The response includes `plant_name` alongside `plant_id` so the UI can display which plant each event belongs to without extra lookups.

Pagination uses cursor-based approach with `before` parameter (the `id` of the last event from the previous page). This is more reliable than offset-based pagination when new events are being added. Default `limit` is 20, maximum 100. An optional `type` query parameter filters by event type (e.g., `?type=watered`).

Response format:
```json
{
  "events": [...],
  "has_more": true
}
```

The `has_more` flag tells the UI whether more events can be loaded. Query fetches `limit + 1` rows, returns `limit` rows, and sets `has_more = true` if the extra row existed. The UI uses infinite scrolling — when the user scrolls near the bottom, the next page is fetched automatically.

**Alternative considered:** Offset-based pagination (`?page=1&per_page=20`) — simpler but produces inconsistent results when events are added between page loads. Cursor-based avoids duplicates and skipped items.

### Decision 4: Auto-log watered event from water_plant handler

When `POST /api/plants/:id/water` is called, insert a care event with `event_type = "watered"` and `occurred_at = datetime('now')` in the same handler, after updating `last_watered`. This keeps the existing water endpoint as the single action for watering. No notes are added to auto-logged events.

This is a simple INSERT after the existing UPDATE — no transaction needed since both are independent writes (a missing care event log is acceptable if it fails; the water action itself already succeeded).

### Decision 5: Care journal timeline in plant detail view

Add a "Care Journal" section below the watering card on the plant detail view. Display events grouped by day (e.g., "Today", "Yesterday", "Feb 10") as a vertical timeline list, newest first, showing:
- Event type icon (droplet for watered, leaf for fertilized, shovel for repotted, scissors for pruned, pencil for custom)
- Event type label
- Relative or absolute date (e.g., "Today", "Yesterday", "Feb 10")
- Notes (if present)
- Delete button (small, subtle)

Limit the initial display to the 20 most recent events. If more exist, show a "Show more" link that loads the rest. This avoids pagination complexity while keeping the UI snappy.

### Decision 6: "Log care" modal/inline form

Add an "+ Add log entry" link below the care journal timeline. Clicking it reveals an inline form (not a modal) with:
- Event type selector (chips/buttons for: Fertilized, Repotted, Pruned, Custom)
- Optional notes text field
- Save button

"Watered" is deliberately excluded from the manual log types — use the existing "Water now" button instead, which also updates `last_watered` and publishes MQTT state.

### Decision 7: New Rust module for care events

Create `src/api/care_events.rs` with handler functions following the existing `plants.rs` patterns:
- `list_care_events` — query with plant_id filter, ordered by `occurred_at` DESC
- `create_care_event` — validate event_type, insert, return created event
- `delete_care_event` — verify plant ownership, delete, return 204

Use `State<SqlitePool>` (via `FromRef<AppState>`) since care event handlers don't need MQTT access.

### Decision 8: CareEvent response struct

```rust
struct CareEvent {
    id: i64,
    plant_id: i64,
    plant_name: String,
    event_type: String,
    notes: Option<String>,
    occurred_at: String,
    created_at: String,
}
```

Include `plant_name` in the response so the global care log can display which plant each event belongs to without extra lookups. The per-plant endpoint also includes it for consistency. Retrieved via a JOIN on the plants table.

## Risks / Trade-offs

- **Per-plant endpoint unpaginated:** The per-plant `GET /api/plants/:id/care` endpoint returns all events without pagination. The UI limits display to 20 with "Show more" client-side. Acceptable for a personal app — individual plants won't accumulate thousands of events. The global endpoint has server-side pagination.
- **No transaction for auto-logged watering:** If the care event INSERT fails after `last_watered` is updated, the watering action succeeds but the journal entry is lost. Acceptable — the primary action (watering) should never fail due to journaling. Log the error and move on.
- **Event type validation at application layer:** Invalid types are caught by the API handler, not the database. A typo in code could insert an invalid type. Mitigated by using a constant set of valid types checked at the handler level.
- **CASCADE delete:** Deleting a plant removes all care events. This matches user expectations — a deleted plant's history is no longer relevant. No soft-delete needed.
