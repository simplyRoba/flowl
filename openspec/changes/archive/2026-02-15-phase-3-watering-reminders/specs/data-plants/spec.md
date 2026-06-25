## MODIFIED Requirements

### Requirement: Plant Database Schema

**ADDED** column: `last_watered` (text, nullable, ISO 8601) to the `plants` table, recording when the plant was last watered.

#### Scenario: Migration adds last_watered column

- **WHEN** the migration runs
- **THEN** the `plants` table has a `last_watered` column of type TEXT, defaulting to NULL

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
