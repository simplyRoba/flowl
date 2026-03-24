## Context

The plant context builder in `src/ai/prompts.rs` currently runs a single SQL query to fetch the 20 most recent care events regardless of type, serializing each as `{ event_type, date, notes }`. Watering events are by far the most frequent care action and dominate this list, pushing out rarer but more informative events (repotting, fertilizing, pruning) that remain diagnostically relevant for years.

The current `PlantContext` struct uses a single `recent_care_events: Vec<CareEventContext>` field. The `CareEventContext` struct has `event_type`, `date`, and optional `notes`.

## Goals / Non-Goals

**Goals:**
- Separate watering history from other care events in the AI context
- Compress watering into a token-efficient date list (1 year window)
- Preserve all non-watering events with full detail (5 year window)
- Keep watering events that carry notes in the full care_events list (they contain useful observations)

**Non-Goals:**
- Changing the system prompt text or AI behavior instructions
- Adding new data to the context (sensor data, plant age, etc.)
- Changing the frontend chat history or API contract shape
- Token counting or context window management

## Decisions

### 1. Two SQL queries instead of one

Run two separate queries with different filters and time windows:

- **Watering dates**: `event_type = 'watered' AND occurred_at >= 1 year ago` (all watering events, regardless of notes)
- **Care events**: `(event_type != 'watered' OR notes IS NOT NULL) AND occurred_at >= 5 years ago`

Watering events with notes appear in both lists: their date goes into `watering_dates` for a complete watering timeline, and the full object goes into `care_events` so the note is visible to the AI.

**Alternative considered**: Single query fetching all events from 5 years, split in Rust. Rejected because it fetches unnecessary watering rows older than 1 year that are immediately discarded. Two targeted queries are clearer and avoid wasted I/O.

### 2. Replace `recent_care_events` with two fields

The `PlantContext` struct changes from:

```rust
recent_care_events: Vec<CareEventContext>,
```

to:

```rust
watering_dates: Vec<String>,    // date-only strings, last 1 year
care_events: Vec<CareEventContext>,  // full objects, last 5 years
```

Both fields use `skip_serializing_if` when empty to keep the JSON compact for plants with no history.

### 3. Time filtering in SQL using SQLite date functions

Use `datetime('now', '-1 year')` and `datetime('now', '-5 years')` in the WHERE clauses. This keeps filtering server-side and works with SQLite's built-in date arithmetic. No Rust-side date filtering needed.

### 4. Order both result sets descending by date

Both queries order by `occurred_at DESC` (most recent first). This puts the most relevant information at the top of the context, which matters for LLM attention patterns.

## Risks / Trade-offs

- **[Unbounded care_events list]** → In practice, non-watering events over 5 years will rarely exceed ~50-100 entries even for active users. The token cost of 100 care event objects is modest (~2-3K tokens). No cap needed.
- **[Breaking JSON shape]** → The `recent_care_events` field disappears from the context JSON. This only affects the system prompt sent to the LLM, not any external API contract. The LLM doesn't have a schema expectation for this field, so there is no compatibility concern.
- **[Watering events with notes appear in both lists]** → Intentional. The `watering_dates` list is a complete watering timeline (all dates, with or without notes). The `care_events` list includes the same watering events when they carry notes, so the AI sees the observation in context. The duplication is minimal and makes both lists self-contained.
