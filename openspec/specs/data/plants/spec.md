## Purpose

Plant entity — database schema, CRUD queries, validation, and photo upload/delete for managing plants.

## Requirements

### Requirement: Plant Database Schema

A `plants` table SHALL store plant entities with the following columns: `id` (integer primary key), `name` (text, required), `species` (text, optional), `icon` (text, default `🪴`), `photo_path` (text, nullable), `location_id` (integer, optional, foreign key to locations), `watering_interval_days` (integer, default 7), `light_needs` (text, default `indirect`), `notes` (text, optional), `created_at` (text, ISO 8601), `updated_at` (text, ISO 8601).

#### Scenario: Table created by migration

- **WHEN** the application starts
- **THEN** the `plants` table exists with all specified columns

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
