## Purpose

Export all user data and photos as a downloadable ZIP archive for backup purposes.

## Requirements

### Requirement: Export all data as ZIP

The system SHALL provide a `GET /api/data/export` endpoint that returns all user data and photos as a ZIP archive download. Thumbnail files (`_200`, `_600`, and `_1000` variants) SHALL be excluded from the archive as they are derived data that can be regenerated.

#### Scenario: Successful export

- **WHEN** a GET request is made to `/api/data/export`
- **THEN** the response has status 200
- **AND** the `Content-Type` header is `application/zip`
- **AND** the `Content-Disposition` header is `attachment; filename="flowl-export-v{version}.zip"` where `{version}` is the server's crate version
- **AND** the body contains a ZIP archive with a `data.json` file at the root

#### Scenario: Export JSON structure

- **WHEN** the `data.json` inside the ZIP is parsed
- **THEN** `version` is a string matching the server's crate version
- **AND** `exported_at` is an ISO 8601 UTC timestamp
- **AND** `locations` is an array of all locations with their `id` and `name`
- **AND** `plants` is an array of all plants with all columns including `location_id`, `photo_path`, `last_watered`, and care info fields (`difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture`)
- **AND** `care_events` is an array of all care events with `plant_id`, `event_type`, `notes`, `photo_path`, `occurred_at`

#### Scenario: Export includes original photos only

- **WHEN** plants or care events have associated photo files
- **THEN** the ZIP archive contains those original photo files under a `photos/` directory
- **AND** each file's name in `photos/` matches the corresponding `photo_path` value
- **AND** thumbnail variants (`_200.jpg`, `_600.jpg`, `_1000.jpg`) SHALL NOT be included in the archive

#### Scenario: Round-trip integrity

- **WHEN** all data is exported, then imported into an empty instance, then exported again
- **THEN** the two `data.json` manifests SHALL be identical except for the `exported_at` timestamp

#### Scenario: Export with no data

- **WHEN** no plants, locations, or care events exist
- **THEN** the response has status 200
- **AND** the `data.json` contains empty arrays for `locations`, `plants`, and `care_events`
- **AND** the ZIP contains no `photos/` entries
