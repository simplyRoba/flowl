## Context

The identify endpoint (`POST /api/ai/identify`) uses OpenAI's structured output with `strict: true` JSON schema. The current schema requires a non-empty `suggestions` array, which forces the AI to always produce plant identifications — even for photos of non-plant objects. The project has an established error contract: all API errors flow through `ApiError` → `{ "code": "...", "message": "..." }` with an appropriate HTTP status. The frontend resolves error codes to localized messages via the `errorCode` i18n map in `resolveError`.

## Goals / Non-Goals

**Goals:**
- Let the AI signal that a photo does not contain a plant, without hallucinating suggestions.
- Surface the rejection to the user through the project's established `ApiError` pattern.

**Non-Goals:**
- General-purpose AI error reporting (only "not a plant" rejection is in scope).
- Supporting custom rejection reasons in the UI (a single localized "not a plant" message is sufficient).

## Decisions

### 1. Extend the AI JSON schema with `rejected` / `rejected_reason`

Add two top-level fields to the structured output schema sent to OpenAI:
- `rejected`: `boolean` (required, defaults to `false` in the prompt instruction)
- `rejected_reason`: `string | null` (required, `null` when `rejected` is `false`)

When `rejected` is `true`, the AI leaves `suggestions` as an empty array and populates `rejected_reason`.

**Why not a single `error` string?** A boolean `rejected` is unambiguous for the AI to set (no risk of it putting a plant name in an error field) and cheaper to check on the backend. The `rejected_reason` carries the AI's free-text explanation for logging/debugging.

**Why not make `suggestions` optional?** OpenAI strict mode requires all fields to be present. Keeping `suggestions` as a required array (that can be empty) is simpler than making it nullable.

### 2. Backend translates rejection into `ApiError::Validation("AI_IDENTIFY_NOT_A_PLANT")`

The `identify` provider method returns `IdentifyResponse` with the new fields. The endpoint handler in `src/api/ai.rs` checks `result.rejected == Some(true)` and returns `ApiError::Validation("AI_IDENTIFY_NOT_A_PLANT")` (HTTP 422).

**Why 422?** The request itself was valid but the content is unprocessable (not a plant). This aligns with how other validation errors work in the project (e.g., `PLANT_INVALID_LIGHT_NEEDS` → 422). Alternatives considered:
- 200 with an error field — breaks the established pattern where 200 always means success with data.
- 400 — implies the request was malformed, which it wasn't.

**Why `ApiError::Validation`?** It maps to 422 and follows the same code/message contract. The `default_message` for `AI_IDENTIFY_NOT_A_PLANT` provides the static English fallback, while the frontend i18n `errorCode` map provides localized messages.

### 3. Frontend uses existing `resolveError` — no new identify state needed

The `IdentifyPanel` catch block already calls `resolveError(e, "identifyPlant")`, which checks `e.code` against the `errorCode` i18n map. Adding `AI_IDENTIFY_NOT_A_PLANT` to the `errorCode` map in all three locales is sufficient — the existing error state in the UI will display the localized message with a retry button.

**Why not a dedicated "not a plant" UI state?** The error state already shows a message + retry button, which is exactly the right UX for this case. A special state would add complexity for no user benefit. The only difference is the message text, which the i18n map handles.

### 4. Prompt instruction tells the AI when to reject

Add a sentence to `build_identify_prompt` instructing the model: if the photo does not show a plant, set `rejected` to `true`, provide a brief reason in `rejected_reason`, and leave `suggestions` empty. This is placed after the existing identification instructions so it reads as a fallback.

## Risks / Trade-offs

- **AI may still hallucinate instead of rejecting** — The structured output guarantees the schema is followed, but the AI could set `rejected: false` and still return low-confidence guesses for ambiguous photos. This is acceptable; the feature targets obvious non-plant cases (shoes, mugs). No mitigation needed beyond the prompt instruction.
- **`rejected_reason` is not shown to the user** — The frontend shows a static localized message, not the AI's free-text reason. This is intentional: the AI's reason is useful for debugging/logging but not reliable enough for user-facing text. The reason is logged on the backend at `warn` level so it's visible in production logs without enabling debug.
- **Schema change requires re-testing with OpenAI strict mode** — Adding fields to a `strict: true` schema can surface edge cases. The `rejected` and `rejected_reason` fields use simple types (`boolean`, `string | null`) that are well-supported.
