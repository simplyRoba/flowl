## 1. Extract prompts module

- [x] 1.1 Create `src/ai/prompts.rs` and register it in `src/ai/mod.rs`
- [x] 1.2 Move `PlantContext`, `CareEventContext`, `PlantContextRow`, `CareEventRow` structs from `src/api/ai.rs` to `src/ai/prompts.rs`
- [x] 1.3 Move `build_plant_context()`, `build_chat_system_prompt()`, `build_summarize_system_prompt()`, and `locale_instruction()` from `src/api/ai.rs` to `src/ai/prompts.rs`
- [x] 1.4 Update `src/api/ai.rs` to import and call the moved functions from `crate::ai::prompts`

## 2. Restructure PlantContext

- [x] 2.1 Create `CurrentState` struct with `watering_status` and `last_watered` fields
- [x] 2.2 Create `CarePreferences` struct with `light_needs`, `watering_interval_days`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture` fields
- [x] 2.3 Update `PlantContext` to use nested `current_state: CurrentState` and `care_preferences: CarePreferences` instead of flat fields
- [x] 2.4 Update `build_plant_context()` to assemble the new nested structure

## 3. Update chat system prompt

- [x] 3.1 Add instruction to the chat system prompt clarifying that `care_preferences` describes desired conditions, not current state

## 4. Tests

- [x] 4.1 Add unit test that `build_plant_context()` serializes with `current_state` and `care_preferences` groups
- [x] 4.2 Add unit test that `build_chat_system_prompt()` includes the care preferences clarification instruction

## 5. Verify

- [x] 5.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
