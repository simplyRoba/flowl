## Purpose

Durable Plant model, CRUD API, validation, and photo upload/delete for managing plants.

## Requirements

### Requirement: Durable Plant Identity

The system SHALL durably preserve each Plant with a generated numeric `id` and required `name`.

#### Scenario: Plant identity available

- **WHEN** the application manages plants
- **THEN** each persisted plant has a generated numeric `id` and required `name`

### Requirement: Durable Plant Attributes

Each durable Plant SHALL preserve `species`, managed-photo association, `location_id`, and `notes` as nullable values; `icon` defaulting to `🪴`; `watering_interval_days` defaulting to 7; `light_needs` defaulting to `indirect`; and ISO 8601 `created_at` and `updated_at` timestamps. When `location_id` is not `null`, it SHALL identify an existing Location.

#### Scenario: Plant attributes available

- **WHEN** the application manages plants
- **THEN** each persisted plant has the nullable, defaulted, and timestamped attributes defined by the durable Plant model

### Requirement: Durable Care Info Values

Each durable Plant SHALL preserve optional care-info values: `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture`. Care-info values MAY be `null` when unset and, when set, SHALL use the allowed values defined in Care Info Enum Validation.

#### Scenario: Care info is unset

- **WHEN** a plant is created without specifying `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, or `soil_moisture`
- **THEN** those care-info values are `null`

### Requirement: Derived Watering History

`last_watered` SHALL be derived from the chronologically latest `watered` Care Event for the Plant and SHALL NOT be an independently authoritative or mutable Plant property.

#### Scenario: Last watered is derived

- **WHEN** a Plant's watering history is evaluated
- **THEN** `last_watered` is derived from its chronologically latest `watered` Care Event

### Requirement: List Plants

The API SHALL return all plants via `GET /api/plants` as a JSON array ordered by name.

#### Scenario: Plants exist

- **WHEN** a GET request is made to `/api/plants`
- **AND** persisted plants exist
- **THEN** the API responds with HTTP 200 and a JSON array of all plants with their location name included

#### Scenario: No plants exist

- **WHEN** a GET request is made to `/api/plants`
- **AND** no persisted plants exist
- **THEN** the API responds with HTTP 200 and an empty JSON array `[]`

### Requirement: Get Plant

The API SHALL return a single plant via `GET /api/plants/:id`.

#### Scenario: Plant found

- **WHEN** a GET request is made to `/api/plants/1`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 200 and the plant JSON object with location name included

#### Scenario: Plant not found

- **WHEN** a GET request is made to `/api/plants/999`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Create Plant

The API SHALL create a new plant via `POST /api/plants` with a JSON body.

#### Scenario: Valid plant created

- **WHEN** a POST request is made to `/api/plants` with `{"name": "Monstera"}`
- **THEN** the API responds with HTTP 201 and the created plant JSON with generated id and timestamps

#### Scenario: Name missing

- **WHEN** a POST request is made to `/api/plants` with `{}`
- **THEN** the API responds with HTTP 422

#### Scenario: Default values applied

- **WHEN** a POST request is made with only `{"name": "Fern"}`
- **THEN** the created plant has `icon` = `🪴`, `watering_interval_days` = 7, `light_needs` = `indirect`
- **AND** `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture` are all null

#### Scenario: Care info fields provided

- **WHEN** a POST request is made with `{"name": "Cactus", "difficulty": "easy", "pet_safety": "safe", "soil_type": "cactus-mix"}`
- **THEN** the created plant has `difficulty` = `easy`, `pet_safety` = `safe`, `soil_type` = `cactus-mix`, `growth_speed` = null

#### Scenario: Invalid care info value

- **WHEN** a POST request is made with `{"name": "Fern", "difficulty": "impossible"}`
- **THEN** the API responds with HTTP 422

### Requirement: Update Plant

The API SHALL update an existing plant via `PUT /api/plants/:id` with a JSON body containing only the fields to update.

#### Scenario: Plant updated

- **WHEN** a PUT request is made to `/api/plants/1` with `{"name": "Monstera Deliciosa"}`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 200 and the updated plant JSON
- **AND** the `updated_at` timestamp is refreshed

#### Scenario: Plant not found

- **WHEN** a PUT request is made to `/api/plants/999`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

#### Scenario: Care info field set

- **WHEN** a PUT request is made to `/api/plants/1` with `{"difficulty": "demanding"}`
- **THEN** the plant's `difficulty` is updated to `demanding`
- **AND** other care info fields remain unchanged

#### Scenario: Care info field cleared

- **WHEN** a PUT request is made to `/api/plants/1` with `{"difficulty": null}`
- **THEN** the plant's `difficulty` is set to `null`

#### Scenario: Invalid care info value on update

- **WHEN** a PUT request is made to `/api/plants/1` with `{"pet_safety": "unknown"}`
- **THEN** the API responds with HTTP 422

### Requirement: Delete Plant

The API SHALL delete a plant via `DELETE /api/plants/:id`.

#### Scenario: Plant deleted

- **WHEN** a DELETE request is made to `/api/plants/1`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 204
- **AND** the plant no longer exists in persisted application data

#### Scenario: Plant not found

- **WHEN** a DELETE request is made to `/api/plants/999`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Photo URL in Response

The plant API response SHALL include a `photo_url` field (string or null) for the plant's associated original managed image.

#### Scenario: Plant has photo

- **WHEN** a plant has an associated original managed image available at `/uploads/abc.jpg`
- **THEN** the response includes `photo_url` = `/uploads/abc.jpg`

#### Scenario: Plant has no photo

- **WHEN** a plant has no associated photo
- **THEN** the response includes `photo_url` = null

### Requirement: Upload Photo

The API SHALL accept a photo upload via `POST /api/plants/:id/photo` as multipart form data.

#### Scenario: Valid upload

- **WHEN** a POST multipart request is made to `/api/plants/1/photo` with a valid supported image under 5 MB
- **THEN** the image is accepted as managed media and associated with the plant
- **AND** the API responds with HTTP 200 and the updated plant JSON

#### Scenario: Replace existing photo

- **WHEN** a photo is uploaded for a plant that already has a photo
- **THEN** the new managed media is associated with the plant
- **AND** the prior associated managed media and its canonical renditions are removed

#### Scenario: Plant not found

- **WHEN** a photo is uploaded to `/api/plants/999/photo`
- **THEN** the API responds with HTTP 404

#### Scenario: Invalid content type

- **WHEN** a file with content type `text/plain` is uploaded
- **THEN** the API responds with HTTP 422

#### Scenario: File too large

- **WHEN** a file exceeding 5 MB is uploaded
- **THEN** the API responds with HTTP 422 and error code `PHOTO_TOO_LARGE`

### Requirement: Delete Photo

The API SHALL delete a plant's photo via `DELETE /api/plants/:id/photo`.

#### Scenario: Photo deleted

- **WHEN** a DELETE request is made to `/api/plants/1/photo`
- **AND** the plant has a photo
- **THEN** the photo association is removed
- **AND** the associated managed media and its canonical renditions are removed
- **AND** the API responds with HTTP 204

#### Scenario: No photo to delete

- **WHEN** a DELETE request is made to `/api/plants/1/photo`
- **AND** the plant has no photo
- **THEN** the API responds with HTTP 404

### Requirement: Photo Cleanup on Plant Deletion

When a plant is deleted, its associated managed media and canonical renditions SHALL be removed. If immediate media removal is unavailable, recovery cleanup SHALL complete the removal without preserving unreferenced media.

#### Scenario: Plant with photo deleted

- **WHEN** a plant with a photo is deleted via `DELETE /api/plants/1`
- **THEN** its associated managed media and canonical renditions are removed or made eligible for recovery cleanup

### Requirement: Plant API Response — Watering Fields

The plant API response SHALL include computed watering fields: `watering_status` (string: `ok`, `due`, or `overdue`), `last_watered` (string or null, ISO 8601), and `next_due` (string or null, ISO 8601 date). The `last_watered` field SHALL be computed as the most recent `occurred_at` from the Plant's Care Events where `event_type = 'watered'`.

#### Scenario: Plant never watered

- **GIVEN** a plant with no `watered` care events and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `due`, `last_watered` = null, `next_due` = null

#### Scenario: Latest watering uses chronological order

- **GIVEN** a plant has watering events recorded with different supported timezone offsets
- **WHEN** the plant is returned from an API endpoint
- **THEN** `last_watered` is the event representing the latest instant regardless of timestamp text ordering

#### Scenario: Plant watered and not yet due

- **GIVEN** a plant with a `watered` care event from yesterday and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `ok`
- **AND** `next_due` = `last_watered` date + 7 days

#### Scenario: Plant due today

- **GIVEN** a plant with a `watered` care event from 7 days ago and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `due`
- **AND** `next_due` = today's date

#### Scenario: Plant overdue

- **GIVEN** a plant with a `watered` care event from 10 days ago and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `overdue`
- **AND** `next_due` = `last_watered` date + 7 days (in the past)

### Requirement: Water Plant

The API SHALL record a watering event via `POST /api/plants/:id/water`.

#### Scenario: Plant watered successfully

- **WHEN** a POST request is made to `/api/plants/1/water`
- **AND** a plant with id 1 exists
- **THEN** a care event with `event_type` = `watered` and `occurred_at` = current datetime is created
- **AND** `updated_at` on the plant is refreshed
- **AND** the API responds with HTTP 200 and the updated plant JSON with recomputed `watering_status` and `last_watered` derived from the new care event

#### Scenario: Plant not found

- **WHEN** a POST request is made to `/api/plants/999/water`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: MQTT Synchronization on Plant State Changes

Successful plant API operations SHALL synchronize or remove the plant's MQTT integration data as defined by `core-mqtt`.

#### Scenario: Plant created

- **WHEN** a new plant is successfully created via `POST /api/plants`
- **THEN** its discovery configuration and current watering state and attributes are synchronized according to `core-mqtt`

#### Scenario: Plant updated

- **WHEN** a plant is successfully updated via `PUT /api/plants/:id`
- **THEN** its discovery configuration and current watering state and attributes are synchronized according to `core-mqtt`

#### Scenario: Plant watered

- **WHEN** a plant is successfully watered via `POST /api/plants/:id/water`
- **THEN** its current watering state and attributes are synchronized according to `core-mqtt`

#### Scenario: Plant deleted

- **WHEN** a plant is successfully deleted via `DELETE /api/plants/:id`
- **THEN** its retained MQTT integration data is removed according to `core-mqtt`

#### Scenario: MQTT connection unavailable

- **WHEN** a successful plant action requires MQTT synchronization or removal
- **AND** MQTT is enabled but not connected
- **THEN** the domain API action completes successfully
- **AND** the MQTT failure is logged
- **AND** the MQTT change remains recoverable through periodic reconciliation, connection-time synchronization, or broker repair, as applicable

### Requirement: Care Info Enum Validation

The API SHALL validate care info fields against their allowed values. Allowed values:
- `difficulty`: `easy`, `moderate`, `demanding`
- `pet_safety`: `safe`, `caution`, `toxic`
- `growth_speed`: `slow`, `moderate`, `fast`
- `soil_type`: `standard`, `cactus-mix`, `orchid-bark`, `peat-moss`
- `soil_moisture`: `dry`, `moderate`, `moist`

`null` is always allowed (the field is optional).

#### Scenario: Valid values accepted

- **WHEN** a plant is created or updated with care info values from the allowed lists
- **THEN** the values are stored as-is

#### Scenario: Invalid value rejected

- **WHEN** a plant is created or updated with a care info value not in the allowed list
- **THEN** the API responds with HTTP 422 and a message identifying the invalid field and value
