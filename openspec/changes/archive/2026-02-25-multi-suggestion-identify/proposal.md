## Why

The AI identify endpoint returns a single plant suggestion per request. In practice, the model frequently returns different species across repeated calls for the same plant photo, forcing users to dismiss and retry until the correct identification appears. Returning 3 ranked suggestions in a single API call lets users browse alternatives immediately and pick the best match.

## What Changes

- The `identify` method on the AI provider trait returns a new `IdentifyResponse` wrapper containing a `Vec<IdentifyResult>` (up to 3 ranked suggestions) instead of a single `IdentifyResult`.
- The OpenAI provider prompt is updated to request the top 3 most likely identifications, and the JSON schema wraps results in a `{ "suggestions": [...] }` envelope.
- `POST /api/ai/identify` returns the new `IdentifyResponse` shape with an array of suggestions.
- The frontend `identifyPlant()` API function returns the new response type.
- The PlantForm suggestion card becomes a carousel with left/right navigation, dot indicators, a "1 / 3" counter, and touch-swipe support on mobile. "Apply to form" applies the currently visible suggestion.
- New i18n keys for suggestion navigation labels in en/de/es.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ai/provider`: The `identify` method return type changes from `IdentifyResult` to `IdentifyResponse { suggestions: Vec<IdentifyResult> }`. The prompt requests 3 ranked suggestions and the JSON schema wraps results in an array envelope.
- `ai/identify`: The endpoint response shape changes from a single `IdentifyResult` object to `{ "suggestions": [ ... ] }` containing up to 3 results.
- `ui/plants`: The identify suggestion card becomes a carousel supporting navigation between multiple suggestions. The `identifyPlant` API client function returns the new response type.

## Impact

- **Backend**: `src/ai/types.rs` (new `IdentifyResponse` type), `src/ai/provider.rs` (trait signature change), `src/ai/openai.rs` (prompt + schema + deserialization), `src/api/ai.rs` (endpoint return type)
- **Frontend**: `ui/src/lib/api.ts` (new `IdentifyResponse` interface, updated function), `ui/src/lib/components/PlantForm.svelte` (carousel state, navigation UI, swipe handling)
- **i18n**: `ui/src/lib/i18n/{en,de,es}.ts` (new navigation keys)
- **No database changes**, no new dependencies, no breaking changes to other features
