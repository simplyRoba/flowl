## Why

The AI chat gives wrong advice because the plant context JSON presents care profile fields (e.g. `soil_moisture: "moist"`) as flat properties alongside current state fields (e.g. `watering_status: "due"`). The AI consistently misreads desired care preferences as current plant conditions — for example, interpreting "soil moisture: moist" as "the soil IS moist" rather than "the plant PREFERS moist soil". Additionally, all system prompt construction and plant context assembly lives in `src/api/ai.rs`, mixing prompt engineering concerns into the HTTP handler layer.

## What Changes

- Restructure the `PlantContext` JSON into semantic groups (`current_state` vs `care_preferences`) so the AI can distinguish what the plant needs from what's happening now
- Add an explicit instruction to the chat system prompt clarifying that care preferences describe desired conditions, not current state
- Extract all prompt-building logic (system prompts, context formatting, locale instructions) from `src/api/ai.rs` into a dedicated `src/ai/prompts.rs` module
- Move the `PlantContext` and `CareEventContext` structs into the prompts module since they exist solely for prompt construction

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ai/chat`: Update plant context builder to group fields into `current_state` and `care_preferences` sections; update chat system prompt to clarify that care preferences are desired conditions
- `ai/summarize`: No requirement changes — the summarize prompt does not use care profile fields. Only affected by the module move (implementation detail, not a spec change).

## Impact

- `src/api/ai.rs` — prompt builder functions and context structs move out
- `src/ai/prompts.rs` — new module receiving the extracted prompt logic
- `src/ai/mod.rs` — register the new `prompts` submodule
- No API contract changes, no database changes, no frontend changes
- AI chat responses will improve in accuracy for plants with care profile data set
