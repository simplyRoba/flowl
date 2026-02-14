## Purpose

Plant entity — database schema, CRUD queries, and validation for managing plants.

## Requirements

### Requirement: Plant Database Schema

A `plants` table SHALL store plant entities with the following columns: `id` (integer primary key), `name` (text, required), `species` (text, optional), `icon` (text, default `🪴`), `location_id` (integer, optional, foreign key to locations), `watering_interval_days` (integer, default 7), `light_needs` (text, default `indirect`), `notes` (text, optional), `created_at` (text, ISO 8601), `updated_at` (text, ISO 8601).

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
