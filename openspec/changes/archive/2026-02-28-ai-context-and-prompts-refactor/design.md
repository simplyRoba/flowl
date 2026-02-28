## Context

The AI chat feature builds a `PlantContext` JSON and embeds it verbatim in the system prompt. All prompt construction and context assembly currently lives in `src/api/ai.rs` — the HTTP handler file. Two problems:

1. Care profile fields (`soil_moisture`, `difficulty`, `light_needs`, etc.) sit at the same level as current state fields (`watering_status`, `last_watered`), causing the AI to misread preferences as conditions.
2. Prompt logic is tangled with HTTP routing, violating separation of concerns.

## Goals / Non-Goals

**Goals:**
- AI correctly distinguishes desired care preferences from current plant state
- Prompt construction logic lives in its own module, decoupled from HTTP handlers
- No API contract changes — same endpoints, same request/response shapes

**Non-Goals:**
- Changing the AI provider trait or OpenAI client implementation
- Modifying the summarize prompt (it doesn't use care profile data)
- Tuning prompt wording beyond the structural fix (can iterate later)

## Decisions

### 1. Restructure PlantContext into semantic groups

Group the flat `PlantContext` struct into nested sections:

```rust
struct PlantContext {
    name: String,
    species: Option<String>,
    location_name: Option<String>,
    notes: Option<String>,
    current_state: CurrentState,
    care_preferences: CarePreferences,
    recent_care_events: Vec<CareEventContext>,
}

struct CurrentState {
    watering_status: String,
    last_watered: Option<String>,
}

struct CarePreferences {
    light_needs: String,
    watering_interval_days: i64,
    difficulty: Option<String>,
    pet_safety: Option<String>,
    growth_speed: Option<String>,
    soil_type: Option<String>,
    soil_moisture: Option<String>,
}
```

**Why nested structs over renaming fields:** Renaming fields (e.g. `preferred_soil_moisture`) would help but still relies on the AI inferring semantics from field names. Grouping under `care_preferences` makes the distinction structural and unambiguous in the JSON. The AI sees a clear object boundary.

**Alternative considered — separate system prompt section:** Could keep flat JSON and add a prompt instruction like "fields X, Y, Z are preferences". Rejected because it's fragile (must be kept in sync with the struct) and the JSON itself would remain misleading.

### 2. Add explicit prompt instruction

Add one line to the chat system prompt:

> "The care_preferences section describes the desired conditions for this plant, not its current state."

Belt-and-suspenders: the JSON structure makes it clear, and the prompt instruction reinforces it for the AI.

### 3. Extract into `src/ai/prompts.rs`

Move these items from `src/api/ai.rs` to a new `src/ai/prompts.rs`:

- `PlantContext`, `CurrentState`, `CarePreferences`, `CareEventContext` structs
- `PlantContextRow`, `CareEventRow` query structs
- `build_plant_context()` function
- `build_chat_system_prompt()` function
- `build_summarize_system_prompt()` function
- `locale_instruction()` helper

The `src/api/ai.rs` handlers call `prompts::build_plant_context()` and `prompts::build_chat_system_prompt()` etc. — the handler only orchestrates, doesn't build prompts.

**Why `src/ai/prompts.rs` over `src/ai/context.rs`:** The module handles both context assembly _and_ prompt construction. "prompts" is the broader, more accurate name. The context structs exist solely to serve prompt building.

**Why keep DB queries in prompts.rs:** `build_plant_context` does SQL queries. Moving just the struct definitions but keeping the query in `api/ai.rs` would split a single concern across two files. Since the query exists only to build prompt context, it belongs with the prompt logic. The function takes `&SqlitePool` as a parameter — no handler coupling.

## Risks / Trade-offs

- **AI behavior change** — Plants with care profile data will get different (better) context. No rollback needed since the old responses were incorrect. → Acceptable, this is the goal.
- **Prompt wording is not localized** — The "care_preferences describes desired conditions" instruction is in English regardless of user locale. → Acceptable; the model instruction layer is always English, only the response language changes.
