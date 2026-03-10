## MODIFIED Requirements

### Requirement: JSON Error Responses

The API SHALL return errors as JSON with a consistent structure containing a `code` field, a `message` field, and an appropriate HTTP status code. The `code` field SHALL be a stable, unique `SCREAMING_SNAKE_CASE` string identifying the error. The `message` field SHALL contain a human-readable English description derived from the code.

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

- **WHEN** an unexpected server-side error occurs (database failure, IO error)
- **THEN** the API responds with HTTP 500 and `{"code": "INTERNAL_ERROR", "message": "..."}`
- **AND** the real error details SHALL be logged server-side
- **AND** internal error details SHALL NOT be exposed to the client

#### Scenario: Conflict

- **WHEN** a request would create a duplicate or violate a uniqueness constraint
- **THEN** the API responds with HTTP 409 and `{"code": "..._ALREADY_EXISTS", "message": "..."}`

#### Scenario: Service unavailable

- **WHEN** a required external service is not configured or not connected
- **THEN** the API responds with HTTP 503 and `{"code": "..._NOT_CONFIGURED" or "..._UNAVAILABLE", "message": "..."}`

## ADDED Requirements

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
- **THEN** the API uses one of: `CARE_EVENT_NOT_FOUND` (404), `CARE_EVENT_TYPE_REQUIRED` (422), `CARE_EVENT_INVALID_TYPE` (422)

#### Scenario: Location errors

- **WHEN** a location operation fails due to client input
- **THEN** the API uses one of: `LOCATION_NOT_FOUND` (404), `LOCATION_NAME_REQUIRED` (422), `LOCATION_ALREADY_EXISTS` (409)

#### Scenario: Photo errors

- **WHEN** a photo operation fails due to client input
- **THEN** the API uses one of: `PHOTO_NOT_FOUND` (404), `PHOTO_NO_FILE` (422), `PHOTO_INVALID_TYPE` (422), `PHOTO_TOO_LARGE` (422)

#### Scenario: Settings errors

- **WHEN** a settings update fails due to client input
- **THEN** the API uses one of: `SETTINGS_INVALID_THEME` (422), `SETTINGS_INVALID_LOCALE` (422)

#### Scenario: Import errors

- **WHEN** an import operation fails due to client input
- **THEN** the API uses one of: `IMPORT_NO_FILE` (400), `IMPORT_INVALID_ARCHIVE` (400), `IMPORT_INVALID_DATA` (400), `IMPORT_VERSION_MISMATCH` (400), `IMPORT_INVALID_FILENAME` (400), `IMPORT_FILE_TOO_LARGE` (400), `IMPORT_VALIDATION_FAILED` (422)

#### Scenario: AI errors

- **WHEN** an AI operation fails
- **THEN** the API uses one of: `AI_NOT_CONFIGURED` (503), `AI_PROVIDER_FAILED` (500), `AI_INVALID_IMAGE` (400), `AI_HISTORY_EMPTY` (422)

#### Scenario: MQTT errors

- **WHEN** an MQTT operation fails
- **THEN** the API uses one of: `MQTT_DISABLED` (409), `MQTT_UNAVAILABLE` (503)

### Requirement: Internal Error Logging

The API SHALL log the original error details for internal failures using `tracing::error!` before returning a generic error response.

#### Scenario: Database error logged

- **WHEN** a database query fails
- **THEN** the original `sqlx::Error` is logged at error level
- **AND** the client receives `{"code": "INTERNAL_ERROR", "message": "..."}` with HTTP 500

#### Scenario: IO error logged

- **WHEN** a file system or IO operation fails internally
- **THEN** the original error is logged at error level
- **AND** the client receives a generic error code with HTTP 500
