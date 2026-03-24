## Why

The strict JSON schema in the identify endpoint forces the AI to always return plant suggestions, even when the photo clearly shows a non-plant object (a coffee mug, a pet, a shoe). This produces hallucinated plant names with low confidence, confusing users and undermining trust in the feature. The AI needs a way to explicitly decline identification.

## What Changes

- Update the JSON schema sent to the AI to include an optional `rejected` boolean and `rejected_reason` string, and allow the `suggestions` array to be empty when the AI declines.
- Update the identify prompt to instruct the AI to set `rejected: true` with a reason when the photo does not contain a plant.
- When the AI returns a rejected response, the backend endpoint translates this into the project's established `ApiError` pattern — returning an appropriate HTTP status with `{ "code": "AI_IDENTIFY_NOT_A_PLANT", "message": "..." }` rather than embedding an error in the success response body.
- Update the frontend identify flow to handle this error code and display a user-facing message instead of the suggestion carousel.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ai/identify`: When the AI declines identification, return the rejection as an `ApiError` using the established `{ "code": "...", "message": "..." }` pattern with an appropriate HTTP status, instead of a 200 with suggestions.
- `ai/provider`: Update the identify JSON schema and prompt to allow the AI to report "not a plant" via a `rejected` flag and reason. The `IdentifyResponse` type gains fields to carry this signal from the AI to the endpoint handler.
- `ui/plant-identify`: Handle the new error code in the identify error state — display the AI's rejection reason to the user.

## Impact

- **Backend types**: `IdentifyResponse` gains `rejected: Option<bool>` and `rejected_reason: Option<String>` fields to carry the AI signal internally.
- **API error**: New `AI_IDENTIFY_NOT_A_PLANT` code added to `ApiError` / `default_message`, returned as a 422 following the same `{ "code", "message" }` contract as all other API errors.
- **JSON schema**: The structured output schema sent to OpenAI adds `rejected` and `rejected_reason` properties and allows an empty `suggestions` array.
- **Prompt**: `build_identify_prompt` adds instruction for the AI to use the rejection fields when the subject is not a plant.
- **API contract**: `POST /api/ai/identify` may now return 422 with `AI_IDENTIFY_NOT_A_PLANT` instead of always returning 200 with suggestions.
- **Frontend**: The identify error handling already catches non-200 responses; it needs to recognize `AI_IDENTIFY_NOT_A_PLANT` and show a distinct "not a plant" message instead of a generic error.
- **Tests**: New backend tests for the rejection path; updated UI tests for the new error state.
