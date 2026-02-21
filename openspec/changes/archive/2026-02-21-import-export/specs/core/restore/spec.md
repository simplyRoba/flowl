## ADDED Requirements

### Requirement: Import data from ZIP
The system SHALL provide a `POST /api/data/import` endpoint that replaces all existing data and photos with the contents of an uploaded ZIP archive.

#### Scenario: Successful import
- **WHEN** a POST request is made to `/api/data/import` with a valid export ZIP archive
- **THEN** the ZIP is fully validated before any existing data is modified (valid ZIP, valid JSON, valid version, valid filenames)
- **AND** all existing locations, plants, and care events are deleted from the database
- **AND** all existing photos are removed from the uploads directory
- **AND** the locations, plants, and care events from `data.json` are inserted
- **AND** photo files from the `photos/` directory in the ZIP are extracted to the uploads directory
- **AND** original timestamps (`created_at`, `updated_at`, `occurred_at`) are preserved
- **AND** the response has status 200 with a summary of imported counts
- **AND** MQTT repair is triggered to clear orphaned retained topics from pre-import plants and republish fresh state for all imported plants

#### Scenario: Import is atomic for database data
- **WHEN** an import is in progress and a database error occurs mid-way
- **THEN** the database changes are rolled back
- **AND** the existing data remains unchanged

#### Scenario: Import body size
- **WHEN** a POST request is made to `/api/data/import`
- **THEN** the endpoint SHALL accept uploads up to 100 MB

#### Scenario: Invalid archive
- **WHEN** the request body is not a valid ZIP or the ZIP is missing `data.json`
- **THEN** the response has status 400 with an error message

#### Scenario: Invalid JSON in archive
- **WHEN** the `data.json` in the ZIP is not valid JSON or is missing required fields (`version`, `locations`, `plants`, `care_events`)
- **THEN** the response has status 400 with an error message

#### Scenario: Version mismatch
- **WHEN** the `data.json` has a `version` whose major or minor component does not match the server's crate version
- **THEN** the response has status 400 with an error message indicating the expected and received versions

#### Scenario: Patch version difference allowed
- **WHEN** the `data.json` has a `version` that differs from the server's crate version only in the patch component
- **THEN** the import proceeds normally

#### Scenario: Path traversal protection
- **WHEN** the ZIP archive contains entries with path traversal sequences (`..`) or absolute paths
- **THEN** those entries are rejected
- **AND** the response has status 400 with an error message
