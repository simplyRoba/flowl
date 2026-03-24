## Why

The AI chat context currently loads the last 20 care events as a flat list regardless of type. Watering events dominate this list (being the most frequent care action), crowding out higher-value events like repotting, fertilizing, and pruning that remain relevant for much longer. This wastes tokens on repetitive low-information entries while losing important long-term care history.

## What Changes

- Split care event context into two structures: a flat `watering_dates` list (last 1 year) and a full `care_events` array (last 5 years)
- Watering events without notes become date-only entries in `watering_dates`
- Watering events with notes are included as full objects in `care_events`
- All non-watering events from the last 5 years are included in `care_events` with no count limit
- Remove the previous hard `LIMIT 20` on care events

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `ai/chat`: The plant context builder changes its care event structure from a single `recent_care_events` array (limited to 20) to a split format with `watering_dates` (date list, 1 year) and `care_events` (full objects, 5 years)

## Impact

- `src/ai/prompts.rs`: `PlantContext` struct and `build_plant_context()` query/logic change
- `src/ai/prompts.rs`: Existing tests for context serialization need updating
- `openspec/specs/ai/chat/spec.md`: Plant context builder requirement and scenarios need updating
