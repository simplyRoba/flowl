## MODIFIED Requirements

### Requirement: Plant Database Schema

A `plants` table SHALL store plant entities with the following columns: `id` (integer primary key), `name` (text, required), `species` (text, optional), `icon` (text, default `🪴`), `photo_path` (text, nullable), `location_id` (integer, optional, foreign key to locations), `watering_interval_days` (integer, default 7), `light_needs` (text, default `indirect`), `difficulty` (text, nullable), `pet_safety` (text, nullable), `growth_speed` (text, nullable), `soil_type` (text, nullable), `soil_moisture` (text, nullable), `notes` (text, optional), `created_at` (text, ISO 8601), `updated_at` (text, ISO 8601).

#### Scenario: Table created by migration

- **WHEN** the application starts
- **THEN** the `plants` table exists with all specified columns including `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture`
- **AND** the `plants` table SHALL NOT contain a `last_watered` column

#### Scenario: Care info columns are nullable

- **WHEN** a plant is created without specifying `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, or `soil_moisture`
- **THEN** those columns SHALL be NULL

### Requirement: Plant API Response — Watering Fields

The plant API response SHALL include computed watering fields: `watering_status` (string: `ok`, `due`, or `overdue`), `last_watered` (string or null, ISO 8601), and `next_due` (string or null, ISO 8601 date). The `last_watered` field SHALL be computed as the most recent `occurred_at` from `care_events` where `event_type = 'watered'` for the plant.

#### Scenario: Plant never watered

- **GIVEN** a plant with no `watered` care events and `watering_interval_days` = 7
- **WHEN** the plant is returned from any API endpoint
- **THEN** `watering_status` = `due`, `last_watered` = null, `next_due` = null

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
