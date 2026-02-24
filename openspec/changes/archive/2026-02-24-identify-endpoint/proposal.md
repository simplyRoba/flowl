## Why

The AI provider's `identify` method is fully implemented but not reachable from the frontend — there is no HTTP endpoint exposing it. This is the next step in the AI integration roadmap (Phase 3) and a prerequisite for the Identify UI (Phase 4). Wiring the endpoint now lets us validate the full identify flow end-to-end via curl before building the UI.

## What Changes

- Add `POST /api/ai/identify` endpoint accepting `multipart/form-data` with one or more photo files
- Extract uploaded images, pass them to the existing `AiProvider::identify` method, and return the `IdentifyResult` as JSON
- Add `Serialize` derive to `IdentifyResult` and `CareProfile` (currently only `Deserialize`) so they can be returned as JSON responses
- Add error handling: return 503 when AI is disabled, 422 when no photos are provided or content type is invalid, and 500 for upstream API failures or unparseable AI responses
- Add an `InternalError` variant to `ApiError` for unexpected failures (AI API errors, deserialization failures)

## Capabilities

### New Capabilities

- `ai/identify`: HTTP endpoint for AI-powered plant identification from uploaded photos

### Modified Capabilities

- `ai/provider`: Add `Serialize` to response types so they can be used in HTTP responses (implementation detail — no behavioral change to the trait itself, but the types gain a new derive)

## Impact

- **Code:** `src/api/ai.rs` (new handler), `src/api/mod.rs` (route registration), `src/ai/types.rs` (add Serialize), `src/api/error.rs` (new InternalError variant)
- **API:** New `POST /api/ai/identify` endpoint
- **Dependencies:** `axum-extra` or `axum::extract::Multipart` for multipart parsing (axum has built-in multipart support, no new crate needed)
- **Tests:** Integration test for the identify endpoint (enabled and disabled states, missing photos)
