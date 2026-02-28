## Purpose

Care events entity — database schema, CRUD API, validation, and global paginated feed for tracking plant care history.

## Requirements

### Requirement: Care Events Database Schema

A `care_events` table SHALL store care event records with the following columns: `id` (integer primary key), `plant_id` (integer, NOT NULL, foreign key to `plants.id` with ON DELETE CASCADE), `event_type` (text, NOT NULL), `notes` (text, nullable), `photo_path` (text, nullable), `occurred_at` (text, NOT NULL, ISO 8601 datetime), `created_at` (text, NOT NULL, default `datetime('now')`).

#### Scenario: Migration creates care_events table

- **WHEN** the migration runs
- **THEN** the `care_events` table exists with all specified columns including `photo_path`
- **AND** a foreign key from `plant_id` to `plants.id` with ON DELETE CASCADE is established

#### Scenario: Cascade delete on plant removal

- **GIVEN** a plant with id 1 has care events
- **WHEN** the plant is deleted via `DELETE /api/plants/1`
- **THEN** all care events with `plant_id` = 1 are automatically deleted

### Requirement: List Care Events

The API SHALL return care events for a plant via `GET /api/plants/:id/care` as a JSON array ordered by `occurred_at` descending.

#### Scenario: Care events exist

- **GIVEN** a plant with id 1 has care events
- **WHEN** a GET request is made to `/api/plants/1/care`
- **THEN** the API responds with HTTP 200 and a JSON array of care events ordered by `occurred_at` descending
- **AND** each event includes the `plant_name`

#### Scenario: No care events

- **GIVEN** a plant with id 1 has no care events
- **WHEN** a GET request is made to `/api/plants/1/care`
- **THEN** the API responds with HTTP 200 and an empty JSON array `[]`

#### Scenario: Plant not found

- **WHEN** a GET request is made to `/api/plants/999/care`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Create Care Event

The API SHALL create a care event via `POST /api/plants/:id/care` with a JSON body.

#### Scenario: Valid care event created

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "fertilized", "notes": "Used liquid fertilizer"}`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 201 and the created care event JSON
- **AND** `occurred_at` defaults to the current datetime
- **AND** `plant_id` is set to 1

#### Scenario: Care event with explicit occurred_at

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "repotted", "occurred_at": "2026-02-14T10:00:00"}`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 201
- **AND** `occurred_at` is set to the provided value

#### Scenario: Invalid event type

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "unknown"}`
- **THEN** the API responds with HTTP 422
- **AND** the error message indicates the valid event types

#### Scenario: Event type missing

- **WHEN** a POST request is made to `/api/plants/1/care` with `{}`
- **THEN** the API responds with HTTP 422

#### Scenario: Watered event triggers MQTT publish

- **WHEN** a care event with `event_type` = `watered` is created for a plant
- **THEN** the plant's watering state and attributes SHALL be published to MQTT

#### Scenario: Non-watered event skips MQTT publish

- **WHEN** a care event with `event_type` other than `watered` is created
- **THEN** no MQTT publish SHALL occur

#### Scenario: Plant not found

- **WHEN** a POST request is made to `/api/plants/999/care` with `{"event_type": "watered"}`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Valid Event Types

The API SHALL accept only the following event types: `watered`, `fertilized`, `repotted`, `pruned`, `custom`, `ai-consultation`.

#### Scenario: Each valid type accepted

- **WHEN** a care event is created with `event_type` set to any of `watered`, `fertilized`, `repotted`, `pruned`, `custom`, `ai-consultation`
- **THEN** the API responds with HTTP 201

#### Scenario: Invalid type rejected

- **WHEN** a care event is created with `event_type` set to `trimmed`
- **THEN** the API responds with HTTP 422

#### Scenario: AI consultation event does not trigger MQTT

- **WHEN** a care event with `event_type` = `ai-consultation` is created
- **THEN** no MQTT publish SHALL occur

### Requirement: Delete Care Event

The API SHALL delete a care event via `DELETE /api/plants/:id/care/:event_id`.

#### Scenario: Care event deleted

- **GIVEN** a care event with id 5 belongs to plant with id 1
- **WHEN** a DELETE request is made to `/api/plants/1/care/5`
- **THEN** the API responds with HTTP 204
- **AND** the care event is removed from the database

#### Scenario: Care event with photo deleted

- **GIVEN** a care event with id 5 has a photo
- **WHEN** a DELETE request is made to `/api/plants/1/care/5`
- **THEN** the photo file SHALL be deleted from disk
- **AND** the care event is removed from the database

#### Scenario: Watered event deletion triggers MQTT publish

- **WHEN** a care event with `event_type` = `watered` is deleted
- **THEN** the plant's watering state and attributes SHALL be republished to MQTT with the updated `last_watered` derived from remaining care events

#### Scenario: Non-watered event deletion skips MQTT publish

- **WHEN** a care event with `event_type` other than `watered` is deleted
- **THEN** no MQTT publish SHALL occur

#### Scenario: Care event not found

- **WHEN** a DELETE request is made to `/api/plants/1/care/999`
- **AND** no care event with id 999 exists for plant 1
- **THEN** the API responds with HTTP 404

#### Scenario: Plant not found

- **WHEN** a DELETE request is made to `/api/plants/999/care/1`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: List All Care Events (Global)

The API SHALL return paginated care events across all plants via `GET /api/care`, ordered by `occurred_at` descending, using cursor-based pagination.

#### Scenario: First page of events

- **WHEN** a GET request is made to `/api/care`
- **THEN** the API responds with HTTP 200 and a JSON object containing an `events` array (up to 20 events) and a `has_more` boolean

#### Scenario: Custom page size

- **WHEN** a GET request is made to `/api/care?limit=5`
- **THEN** the API responds with at most 5 events

#### Scenario: Next page via cursor

- **GIVEN** a previous response contained an event with id 42 as the last item
- **WHEN** a GET request is made to `/api/care?before=42`
- **THEN** the API responds with events older than the event with id 42

#### Scenario: Filter by event type

- **WHEN** a GET request is made to `/api/care?type=watered`
- **THEN** the API responds with only care events of type `watered`
- **AND** pagination and `has_more` apply to the filtered set

#### Scenario: Invalid filter type

- **WHEN** a GET request is made to `/api/care?type=invalid`
- **THEN** the API responds with HTTP 422

#### Scenario: No more events

- **WHEN** all events have been fetched
- **THEN** `has_more` is `false`

#### Scenario: No events exist

- **WHEN** a GET request is made to `/api/care`
- **AND** no care events exist
- **THEN** the API responds with HTTP 200, an empty `events` array, and `has_more` = `false`

### Requirement: Care Event Response Format

The care event API response SHALL include: `id` (number), `plant_id` (number), `plant_name` (string), `event_type` (string), `notes` (string or null), `photo_url` (string or null), `occurred_at` (string, ISO 8601), `created_at` (string, ISO 8601).

#### Scenario: Full care event response

- **WHEN** a care event is returned from any API endpoint
- **THEN** the response includes all specified fields including `plant_name` and `photo_url`

#### Scenario: Care event with photo

- **WHEN** a care event has `photo_path` = `abc.jpg`
- **THEN** the response includes `photo_url` = `/uploads/abc.jpg`

#### Scenario: Care event without photo

- **WHEN** a care event has `photo_path` = NULL
- **THEN** the response includes `photo_url` = null

### Requirement: Upload Care Event Photo

The API SHALL accept a photo upload via `POST /api/plants/:id/care/:event_id/photo` as multipart form data.

#### Scenario: Valid upload

- **WHEN** a POST multipart request is made to `/api/plants/1/care/5/photo` with a JPEG file under 5 MB
- **AND** care event 5 belongs to plant 1
- **THEN** the file is saved to the upload directory with a UUID filename
- **AND** the care event's `photo_path` is updated
- **AND** the API responds with HTTP 200 and the updated care event JSON

#### Scenario: Replace existing photo

- **WHEN** a photo is uploaded for a care event that already has a photo
- **THEN** the old photo file is deleted from disk
- **AND** the new photo is saved and `photo_path` is updated

#### Scenario: Care event not found

- **WHEN** a photo is uploaded to `/api/plants/1/care/999/photo`
- **AND** no care event with id 999 exists for plant 1
- **THEN** the API responds with HTTP 404

#### Scenario: Invalid content type

- **WHEN** a file with content type `text/plain` is uploaded
- **THEN** the API responds with HTTP 422

#### Scenario: File too large

- **WHEN** a file exceeding 5 MB is uploaded
- **THEN** the API responds with HTTP 422

### Requirement: Delete Care Event Photo

The API SHALL delete a care event's photo via `DELETE /api/plants/:id/care/:event_id/photo`.

#### Scenario: Photo deleted

- **WHEN** a DELETE request is made to `/api/plants/1/care/5/photo`
- **AND** the care event has a photo
- **THEN** the file is deleted from disk
- **AND** `photo_path` is set to NULL
- **AND** the API responds with HTTP 204

#### Scenario: No photo to delete

- **WHEN** a DELETE request is made to `/api/plants/1/care/5/photo`
- **AND** the care event has no photo
- **THEN** the API responds with HTTP 404

### Requirement: Care Event Photo Cleanup on Plant Deletion

When a plant is deleted, care event rows are removed by CASCADE. Any orphaned care event photo files on disk SHALL be cleaned up by the startup orphan cleanup (see `core/image-store`).

#### Scenario: Plant with care event photos deleted

- **WHEN** a plant with care events that have photos is deleted via `DELETE /api/plants/1`
- **THEN** the CASCADE removes care event rows
- **AND** orphaned photo files are deleted on the next application startup
