## Why

The UI shows raw English error strings from the backend regardless of the active locale. Store error handlers use `e instanceof Error ? e.message : t.error.xxx` -- the first branch bypasses i18n entirely. One workaround exists (`localizeLocationError` regex-matches `"Location 'x' already exists"`), but it doesn't scale.

With the companion `backend-error-codes` change adding stable `code` fields to every API error response, the frontend can now map error codes to i18n translation keys instead of displaying raw messages.

## What Changes

- Parse the `code` field from API error responses in `api.ts`, storing it on `ApiError`.
- Add an `errorCode` group to i18n dictionaries (en, de, es) mapping error codes to localized messages.
- Replace the `e.message` pattern in all stores with a helper that resolves `ApiError.code` to the matching i18n key.
- Remove the `localizeLocationError` regex hack.
- For unknown codes, fall back to a generic translated error message rather than showing raw English.

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `ui/i18n`: Add an `errorCode` translation group mapping backend error codes to localized messages.

## Impact

- `ui/src/lib/api.ts`: `ApiError` class gains `code` field, response parsing updated
- `ui/src/lib/i18n/en.ts`, `de.ts`, `es.ts`: New `errorCode` translation group
- `ui/src/lib/stores/plants.ts`, `care.ts`, `locations.ts`: Error handling updated to use code-based resolution
- `ui/src/lib/stores/locations.ts`: `localizeLocationError` removed
- Store and API tests updated
