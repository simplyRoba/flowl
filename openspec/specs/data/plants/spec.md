## Purpose

Plant entity — database schema, CRUD queries, validation, and photo upload/delete for managing plants.

## Requirements

### Requirement: Plant Database Schema

A `plants` table SHALL store plant entities with the following columns: `id` (integer primary key), `name` (text, required), `species` (text, optional), `icon` (text, default `🪴`), `photo_path` (text, nullable), `location_id` (integer, optional, foreign key to locations), `watering_interval_days` (integer, default 7), `last_watered` (text, nullable, ISO 8601), `light_needs` (text, default `indirect`), `difficulty` (text, nullable), `pet_safety` (text, nullable), `growth_speed` (text, nullable), `soil_type` (text, nullable), `soil_moisture` (text, nullable), `notes` (text, optional), `created_at` (text, ISO 8601), `updated_at` (text, ISO 8601).

#### Scenario: Table created by migration

- **WHEN** the application starts
- **THEN** the `plants` table exists with all specified columns including `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture`

#### Scenario: Care info columns are nullable

- **WHEN** a plant is created without specifying `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, or `soil_moisture`
- **THEN** those columns SHALL be NULL

### Requirement: List Plants

The API SHALL return all plants via `GET /api/plants` as a JSON array ordered by name.

#### Scenario: Plants exist

- **WHEN** a GET request is made to `/api/plants`
- **AND** plants exist in the database
- **THEN** the API responds with HTTP 200 and a JSON array of all plants with their location name included

#### Scenario: No plants exist

- **WHEN** a GET request is made to `/api/plants`
- **AND** no plants exist in the database
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
- **THEN** the plant's `difficulty` is set to NULL

#### Scenario: Invalid care info value on update

- **WHEN** a PUT request is made to `/api/plants/1` with `{"pet_safety": "unknown"}`
- **THEN** the API responds with HTTP 422

### Requirement: Delete Plant

The API SHALL delete a plant via `DELETE /api/plants/:id`.

#### Scenario: Plant deleted

- **WHEN** a DELETE request is made to `/api/plants/1`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 204
- **AND** the plant is removed from the database

#### Scenario: Plant not found

- **WHEN** a DELETE request is made to `/api/plants/999`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Photo URL in Response

The plant API response SHALL include a `photo_url` field (string or null) computed from `photo_path`.

#### Scenario: Plant has photo

- **WHEN** a plant has `photo_path` = `abc.jpg`
- **THEN** the response includes `photo_url` = `/uploads/abc.jpg`

#### Scenario: Plant has no photo

- **WHEN** a plant has `photo_path` = NULL
- **THEN** the response includes `photo_url` = null

### Requirement: Upload Photo

The API SHALL accept a photo upload via `POST /api/plants/:id/photo` as multipart form data.

#### Scenario: Valid upload

- **WHEN** a POST multipart request is made to `/api/plants/1/photo` with a JPEG file under 5 MB
- **THEN** the file is saved to the upload directory with a UUID filename
- **AND** the plant's `photo_path` is updated
- **AND** the API responds with HTTP 200 and the updated plant JSON

#### Scenario: Replace existing photo

- **WHEN** a photo is uploaded for a plant that already has a photo
- **THEN** the old photo file is deleted from disk
- **AND** the new photo is saved and `photo_path` is updated

#### Scenario: Plant not found

- **WHEN** a photo is uploaded to `/api/plants/999/photo`
- **THEN** the API responds with HTTP 404

#### Scenario: Invalid content type

- **WHEN** a file with content type `text/plain` is uploaded
- **THEN** the API responds with HTTP 422

#### Scenario: File too large

- **WHEN** a file exceeding 5 MB is uploaded
- **THEN** the API responds with HTTP 422

### Requirement: Delete Photo

The API SHALL delete a plant's photo via `DELETE /api/plants/:id/photo`.

#### Scenario: Photo deleted

- **WHEN** a DELETE request is made to `/api/plants/1/photo`
- **AND** the plant has a photo
- **THEN** the file is deleted from disk
- **AND** `photo_path` is set to NULL
- **AND** the API responds with HTTP 204

#### Scenario: No photo to delete

- **WHEN** a DELETE request is made to `/api/plants/1/photo`
- **AND** the plant has no photo
- **THEN** the API responds with HTTP 404

### Requirement: Photo Cleanup on Plant Deletion

When a plant is deleted, its photo file (if any) SHALL be deleted from disk.

#### Scenario: Plant with photo deleted

- **WHEN** a plant with a photo is deleted via `DELETE /api/plants/1`
- **THEN** the photo file is removed from the upload directory

### Requirement: Plant API Response — Watering Fields

The plant API response SHALL include computed watering fields: `watering_status` (string: `ok`, `due`, or `overdue`), `last_watered` (string or null, ISO 8601), and `next_due` (string or null, ISO 8601 date).

#### Scenario: Plant never watered

- **GIVEN** a plant with `last_watered` = NULL and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `due`, `last_watered` = null, `next_due` = null

#### Scenario: Plant watered and not yet due

- **GIVEN** a plant with `last_watered` = yesterday and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `ok`
- **AND** `next_due` = `last_watered` date + 7 days

#### Scenario: Plant due today

- **GIVEN** a plant with `last_watered` = 7 days ago and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `due`
- **AND** `next_due` = today's date

#### Scenario: Plant overdue

- **GIVEN** a plant with `last_watered` = 10 days ago and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `overdue`
- **AND** `next_due` = `last_watered` date + 7 days (in the past)

### Requirement: Water Plant

The API SHALL record a watering event via `POST /api/plants/:id/water`.

#### Scenario: Plant watered successfully

- **WHEN** a POST request is made to `/api/plants/1/water`
- **AND** a plant with id 1 exists
- **THEN** the plant's `last_watered` is set to the current datetime
- **AND** `updated_at` is refreshed
- **AND** the API responds with HTTP 200 and the updated plant JSON with recomputed `watering_status`
- **AND** a care event with `event_type` = `watered` and `occurred_at` = current datetime is automatically created

#### Scenario: Plant not found

- **WHEN** a POST request is made to `/api/plants/999/water`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: MQTT Publishing on Plant State Changes

The plant API handlers SHALL trigger MQTT publishing when plant state changes.

#### Scenario: Plant created

- **WHEN** a new plant is created via `POST /api/plants`
- **THEN** an MQTT auto-discovery config is published for the plant
- **AND** the initial watering state is published
- **AND** watering attributes (next_due, last_watered, watering_interval_days) are published

#### Scenario: Plant updated

- **WHEN** a plant is updated via `PUT /api/plants/:id`
- **THEN** the MQTT auto-discovery config is re-published (name may have changed)
- **AND** the watering state is published
- **AND** watering attributes are published

#### Scenario: Plant watered

- **WHEN** a plant is watered via `POST /api/plants/:id/water`
- **THEN** the watering state is published to MQTT
- **AND** watering attributes are published

#### Scenario: Plant deleted

- **WHEN** a plant is deleted via `DELETE /api/plants/:id`
- **THEN** empty payloads are published to the plant's MQTT discovery, state, and attributes topics to remove it from Home Assistant

#### Scenario: MQTT unavailable

- **WHEN** any plant action triggers MQTT publishing
- **AND** the MQTT client is not connected
- **THEN** the API action completes successfully
- **AND** the MQTT error is logged

### Requirement: Care Info Enum Validation

The API SHALL validate care info fields against their allowed values. Allowed values:
- `difficulty`: `easy`, `moderate`, `demanding`
- `pet_safety`: `safe`, `caution`, `toxic`
- `growth_speed`: `slow`, `moderate`, `fast`
- `soil_type`: `standard`, `cactus-mix`, `orchid-bark`, `peat-moss`
- `soil_moisture`: `dry`, `moderate`, `moist`

NULL is always allowed (field is optional).

#### Scenario: Valid values accepted

- **WHEN** a plant is created or updated with care info values from the allowed lists
- **THEN** the values are stored as-is

#### Scenario: Invalid value rejected

- **WHEN** a plant is created or updated with a care info value not in the allowed list
- **THEN** the API responds with HTTP 422 and a message identifying the invalid field and value
