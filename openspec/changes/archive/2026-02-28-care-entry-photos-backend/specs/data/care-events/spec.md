## MODIFIED Requirements

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

## ADDED Requirements

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
