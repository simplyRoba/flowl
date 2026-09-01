## Purpose

REST API layer providing JSON request/response handling, error responses, and route mounting under the `/api` prefix.

## Requirements

### Requirement: API Router

The application SHALL mount all REST API routes under the `/api` prefix on the Axum server.

#### Scenario: API route accessible

- **WHEN** a request is made to `/api/plants`
- **THEN** the API router handles the request

#### Scenario: Non-API route falls through

- **WHEN** a request is made to a path not starting with `/api`
- **THEN** the request falls through to the SPA static file handler

### Requirement: JSON Error Responses

Errors returned by Flowl application code through `ApiError`, including errors from handlers, middleware, and custom extractors, SHALL use JSON with a consistent structure containing a `code` field, a `message` field, and an appropriate HTTP status code. The `code` field SHALL be a stable, unique `SCREAMING_SNAKE_CASE` string identifying the error. The `message` field SHALL contain a human-readable English description derived from the code. Framework-generated rejections that bypass `ApiError`, including malformed path or query extraction and unmatched routes, are outside this requirement.

#### Scenario: Validation error

- **WHEN** a request body is missing required fields
- **THEN** the API responds with HTTP 422 and `{"code": "...", "message": "..."}`
- **AND** the `code` uniquely identifies the specific validation failure (e.g., `PLANT_NAME_REQUIRED`, `CARE_EVENT_TYPE_REQUIRED`)

#### Scenario: Not found

- **WHEN** a request references a resource that does not exist
- **THEN** the API responds with HTTP 404 and `{"code": "..._NOT_FOUND", "message": "..."}`

#### Scenario: Invalid JSON body

- **WHEN** a request body contains invalid JSON
- **THEN** the API responds with HTTP 400 and `{"code": "INVALID_REQUEST_BODY", "message": "..."}`

#### Scenario: Internal failure

- **WHEN** an unexpected server-side error prevents the requested logical state change (database failure, required IO error)
- **THEN** the API responds with HTTP 500 and `{"code": "INTERNAL_ERROR", "message": "..."}`
- **AND** the real error details SHALL be logged server-side
- **AND** internal error details SHALL NOT be exposed to the client

#### Scenario: Best-effort file cleanup after logical deletion

- **GIVEN** a deletion has removed the file reference from the database
- **WHEN** physical file cleanup fails unexpectedly
- **THEN** the original deletion response remains successful
- **AND** the filesystem error is logged at error level
- **AND** startup orphan cleanup can retry removal

#### Scenario: Rate limit exceeded

- **WHEN** an AI endpoint receives a request that exceeds the configured rate limit
- **THEN** the API responds with HTTP 429 and `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be forwarded to the AI provider

#### Scenario: Conflict

- **WHEN** a request would create a duplicate or violate a uniqueness constraint
- **THEN** the API responds with HTTP 409 and `{"code": "..._ALREADY_EXISTS", "message": "..."}`

#### Scenario: Service unavailable

- **WHEN** a required external service is not configured or not connected
- **THEN** the API responds with HTTP 503 and `{"code": "..._NOT_CONFIGURED" or "..._UNAVAILABLE", "message": "..."}`

### Requirement: Error Code Catalog

The API SHALL define a fixed catalog of error codes. Each error code SHALL map to exactly one HTTP status code and one default message.

#### Scenario: Generic errors

- **WHEN** a generic error occurs
- **THEN** the API uses one of: `INTERNAL_ERROR` (500), `INVALID_REQUEST_BODY` (400)

#### Scenario: Plant errors

- **WHEN** a plant operation fails due to client input
- **THEN** the API uses one of: `PLANT_NOT_FOUND` (404), `PLANT_NAME_REQUIRED` (422), `PLANT_INVALID_LIGHT_NEEDS` (422), `PLANT_INVALID_DIFFICULTY` (422), `PLANT_INVALID_PET_SAFETY` (422), `PLANT_INVALID_GROWTH_SPEED` (422), `PLANT_INVALID_SOIL_TYPE` (422), `PLANT_INVALID_SOIL_MOISTURE` (422), `PLANT_INVALID_WATERING_INTERVAL` (422)

#### Scenario: Care event errors

- **WHEN** a care event operation fails due to client input
- **THEN** the API uses one of: `CARE_EVENT_NOT_FOUND` (404), `CARE_EVENT_TYPE_REQUIRED` (422), `CARE_EVENT_INVALID_TYPE` (422), `CARE_EVENT_NOTES_REQUIRED` (422), `CARE_EVENT_OCCURRED_AT_REQUIRED` (422), `CARE_EVENT_INVALID_OCCURRED_AT` (422), `CARE_EVENT_INVALID_CURSOR` (422)

#### Scenario: Location errors

- **WHEN** a location operation fails due to client input
- **THEN** the API uses one of: `LOCATION_NOT_FOUND` (404), `LOCATION_NAME_REQUIRED` (422), `LOCATION_ALREADY_EXISTS` (409)

#### Scenario: Photo errors

- **WHEN** a photo operation or multipart photo upload fails due to client input
- **THEN** the API uses one of: `PHOTO_NOT_FOUND` (404), `PHOTO_NO_FILE` (422), `PHOTO_INVALID_TYPE` (422), `PHOTO_TOO_LARGE` (422), `PHOTO_SAVE_FAILED` (500)

#### Scenario: Identify multipart photo errors

- **WHEN** an AI identification request has no `photos` files or has an unsupported photo content type
- **THEN** the API uses `PHOTO_NO_FILE` (422) or `PHOTO_INVALID_TYPE` (422), respectively

#### Scenario: Settings errors

- **WHEN** a settings update fails due to client input
- **THEN** the API uses one of: `SETTINGS_INVALID_THEME` (422), `SETTINGS_INVALID_LOCALE` (422)

#### Scenario: Import errors

- **WHEN** an import operation fails due to client input
- **THEN** the API uses one of: `IMPORT_NO_FILE` (400), `IMPORT_INVALID_ARCHIVE` (400), `IMPORT_INVALID_DATA` (400), `IMPORT_VERSION_MISMATCH` (400), `IMPORT_INVALID_FILENAME` (400), `IMPORT_FILE_TOO_LARGE` (400), `IMPORT_VALIDATION_FAILED` (422)

#### Scenario: AI errors

- **WHEN** AI-specific validation, provider processing, or response streaming fails after request parsing
- **THEN** the API uses one of: `AI_NOT_CONFIGURED` (503), `AI_PROVIDER_FAILED` (500), `AI_INVALID_IMAGE` (400), `AI_HISTORY_EMPTY` (422), `AI_RATE_LIMITED` (429), `AI_IDENTIFY_NOT_A_PLANT` (422), `AI_TOO_MANY_IMAGES` (422), `AI_STREAM_ERROR` (500)

#### Scenario: MQTT errors

- **WHEN** an MQTT operation fails
- **THEN** the API uses one of: `MQTT_DISABLED` (409), `MQTT_UNAVAILABLE` (503)

### Requirement: Internal Error Logging

The API SHALL log the original error details for internal failures using `tracing::error!` before returning a generic error response.

#### Scenario: Database error logged

- **WHEN** a database query fails
- **THEN** the original `sqlx::Error` is logged at error level
- **AND** the client receives `{"code": "INTERNAL_ERROR", "message": "..."}` with HTTP 500

#### Scenario: Required IO error logged

- **WHEN** a filesystem or IO operation required to complete the requested logical state change fails
- **THEN** the original error is logged at error level
- **AND** the client receives a generic error code with HTTP 500

### Requirement: API authentication-required response

When OIDC authentication is enabled, every unauthenticated request under `/api/*` SHALL return HTTP 401 in the existing JSON error format with code `AUTHENTICATION_REQUIRED` and its fixed safe default message. API authentication failures SHALL include `Cache-Control: no-store` and SHALL NOT redirect to `/login`, return SPA HTML, or redirect to the OIDC provider.

#### Scenario: Unauthenticated API request

- **WHEN** authentication is enabled
- **AND** a request without a valid authenticated session is made to any `/api/*` route
- **THEN** the API responds with HTTP 401
- **AND** the body is `{"code":"AUTHENTICATION_REQUIRED","message":"Authentication is required"}`
- **AND** the response includes `Cache-Control: no-store`

#### Scenario: API request is never redirected for login

- **WHEN** an unauthenticated API request would otherwise encounter the browser login guard
- **THEN** the response remains the JSON `AUTHENTICATION_REQUIRED` error
- **AND** has no redirect to HTML or an OIDC endpoint

#### Scenario: Arbitrary unauthorized error remains distinct

- **WHEN** an API response has HTTP 401 for a reason other than a missing or expired Flowl session
- **THEN** it SHALL NOT use `AUTHENTICATION_REQUIRED`

#### Scenario: Disabled mode has no authentication error

- **WHEN** authentication is disabled
- **THEN** the API does not require a session
- **AND** existing endpoint status and JSON behavior remains unchanged
