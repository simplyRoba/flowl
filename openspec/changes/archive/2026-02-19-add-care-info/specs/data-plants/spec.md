## MODIFIED Requirements

### Requirement: Plant Database Schema

A `plants` table SHALL store plant entities with the following columns: `id` (integer primary key), `name` (text, required), `species` (text, optional), `icon` (text, default `🪴`), `photo_path` (text, nullable), `location_id` (integer, optional, foreign key to locations), `watering_interval_days` (integer, default 7), `last_watered` (text, nullable, ISO 8601), `light_needs` (text, default `indirect`), `difficulty` (text, nullable), `pet_safety` (text, nullable), `growth_speed` (text, nullable), `soil_type` (text, nullable), `soil_moisture` (text, nullable), `notes` (text, optional), `created_at` (text, ISO 8601), `updated_at` (text, ISO 8601).

#### Scenario: Table created by migration

- **WHEN** the application starts
- **THEN** the `plants` table exists with all specified columns including `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture`

#### Scenario: Care info columns are nullable

- **WHEN** a plant is created without specifying `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, or `soil_moisture`
- **THEN** those columns SHALL be NULL

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

## ADDED Requirements

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
