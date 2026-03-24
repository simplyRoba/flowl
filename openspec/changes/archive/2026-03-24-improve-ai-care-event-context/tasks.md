## 1. Update PlantContext struct

- [x] 1.1 Replace `recent_care_events: Vec<CareEventContext>` with `watering_dates: Vec<String>` and `care_events: Vec<CareEventContext>` in the `PlantContext` struct, both with `#[serde(skip_serializing_if = "Vec::is_empty")]`

## 2. Update build_plant_context query and logic

- [x] 2.1 Replace the single care events query with two queries: watering dates (last 1 year, all watering events ordered DESC) and care events (last 5 years, non-watering events + watering events with notes, ordered DESC)
- [x] 2.2 Map watering dates query results to a `Vec<String>` of date-only strings
- [x] 2.3 Map care events query results to `Vec<CareEventContext>` as before
- [x] 2.4 Wire both into the `PlantContext` constructor

## 3. Update tests

- [x] 3.1 Update `build_plant_context_serializes_with_nested_groups` test to use `watering_dates` and `care_events` instead of `recent_care_events`
- [x] 3.2 Update `chat_system_prompt_contains_plant_context` test to assert on the new fields
- [x] 3.3 Update `chat_system_prompt_no_optional_fields` test to verify empty collections are omitted
- [x] 3.4 Add a test verifying watering events with notes appear in both `watering_dates` and `care_events`

## 4. Verify

- [x] 4.1 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`
