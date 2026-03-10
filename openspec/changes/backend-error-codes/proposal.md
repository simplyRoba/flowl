## Why

Every `sqlx` error in the API is mapped to `ApiError::BadRequest(e.to_string())`, which returns HTTP 400 with raw database error messages leaked to clients. Internal failures (connection loss, constraint violations) should be 500s, not 400s. Additionally, error responses carry only a free-form English `message` string, making it impossible for the frontend to reliably map errors to localized UI text. The frontend currently regex-matches error strings or shows them raw regardless of locale.

Introducing stable, unique error codes in every API error response solves both problems: the backend can separate client errors from internal failures, and the frontend gets a machine-readable code to key i18n translations on.

## What Changes

- **BREAKING**: Error response shape changes from `{"message": "..."}` to `{"code": "ERROR_CODE", "message": "..."}`. The `message` field remains as a human-readable English fallback.
- Replace all `ApiError::BadRequest(e.to_string())` on `sqlx` errors with `ApiError::InternalError("INTERNAL_ERROR")`, logging the real error server-side.
- Change `ApiError` variants to carry a static error code (`&'static str`) instead of a free-form `String`.
- Derive the human-readable `message` from the error code via a lookup function.
- Categorize every error site in the codebase with a specific, unique error code.

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `core/api`: Change JSON error response structure to include a `code` field alongside `message`, and require each error variant to carry a static error code.

## Impact

- `src/api/error.rs`: Restructured `ApiError` enum and `IntoResponse` implementation
- `src/api/plants.rs`, `src/api/care_events.rs`, `src/api/locations.rs`, `src/api/photos.rs`, `src/api/settings.rs`, `src/api/backup.rs`, `src/api/restore.rs`, `src/api/ai.rs`, `src/api/mqtt.rs`, `src/api/stats.rs`: All error call sites updated
- All integration tests: Assertions on error responses updated to check `code` field
- Frontend `api.ts`: Must parse new `code` field (covered by companion change `frontend-error-mapping`)
