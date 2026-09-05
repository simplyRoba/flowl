## Purpose

Import user data and photos from a previously exported ZIP archive, replacing all existing data.

## Requirements

### Requirement: Import data from ZIP

The system SHALL provide a `POST /api/data/import` endpoint that replaces all existing logical data and managed media with the contents of an uploaded ZIP archive. The archive SHALL be fully validated before replacement begins. After imported originals are available, the system SHALL regenerate the canonical derived renditions defined by `core-image-store`.

#### Scenario: Successful import

- **WHEN** a POST request is made to `/api/data/import` with a valid export ZIP archive
- **THEN** the ZIP is fully validated before existing logical data or managed media is replaced, including ZIP validity, JSON validity, version compatibility, and valid archive filenames
- **AND** all existing locations, plants, care events, and managed media are logically replaced by the archive contents
- **AND** imported original photos are associated with the imported data
- **AND** canonical derived renditions are regenerated for all imported photos
- **AND** original timestamps (`created_at`, `updated_at`, `occurred_at`) are preserved
- **AND** the response has status 200 with a summary of imported counts
- **AND** MQTT repair is triggered to clear orphaned retained topics from pre-import plants and republish fresh state for all imported plants

#### Scenario: Replacement failure preserves prior logical data

- **WHEN** an import fails during replacement
- **THEN** the existing logical data remains unchanged
- **AND** any staged imported media that remains unreferenced is removed during media recovery at the next application startup

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
- **THEN** the response has status 400 with error code `IMPORT_VERSION_MISMATCH`

#### Scenario: Patch version difference allowed

- **WHEN** the `data.json` has a valid numeric `major.minor.patch` version that differs from the server's crate version only in the patch component
- **THEN** the import proceeds normally

#### Scenario: Malformed version rejected

- **WHEN** the `data.json` version is not exactly three numeric `major.minor.patch` components
- **THEN** the response has status 400 with error code `IMPORT_VERSION_MISMATCH`

#### Scenario: Path traversal protection

- **WHEN** the ZIP archive contains entries with path traversal sequences (`..`) or absolute paths
- **THEN** those entries are rejected
- **AND** the response has status 400 with an error message
