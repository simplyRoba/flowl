## Purpose

Plant entity — MODIFIED to add `photo_path` column and photo upload/delete endpoints.

## Requirements

### Requirement: Photo Storage Column

The `plants` table SHALL have a `photo_path` column (TEXT, nullable) storing the filename of an uploaded photo.

#### Scenario: Column added by migration

- **WHEN** the migration runs
- **THEN** the `plants` table has a `photo_path` column that is nullable

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
