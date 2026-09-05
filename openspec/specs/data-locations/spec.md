## Purpose

Durable Location model, APIs, and plant count for reusable room/location labels.

## Requirements

### Requirement: Durable Location Model

The system SHALL durably preserve each reusable Location with a generated numeric `id` and a required unique `name`.

#### Scenario: Location model available

- **WHEN** the application manages locations
- **THEN** each persisted location has a generated numeric `id` and a unique required `name`

### Requirement: List Locations

The API SHALL return all locations via `GET /api/locations` as a JSON array ordered by name.

#### Scenario: Locations exist

- **WHEN** a GET request is made to `/api/locations`
- **THEN** the API responds with HTTP 200 and a JSON array of all locations

#### Scenario: No locations exist

- **WHEN** a GET request is made to `/api/locations`
- **AND** no locations exist
- **THEN** the API responds with HTTP 200 and an empty JSON array `[]`

### Requirement: Create Location

The API SHALL create a new location via `POST /api/locations` with a JSON body.

#### Scenario: Valid location created

- **WHEN** a POST request is made to `/api/locations` with `{"name": "Living Room"}`
- **THEN** the API responds with HTTP 201 and the created location JSON

#### Scenario: Name missing

- **WHEN** a POST request is made to `/api/locations` with `{}`
- **THEN** the API responds with HTTP 422

#### Scenario: Duplicate name

- **WHEN** a POST request is made to `/api/locations` with `{"name": "Living Room"}`
- **AND** a location named "Living Room" already exists
- **THEN** the API responds with HTTP 409 and `{"message": "..."}`

### Requirement: Update Location

The API SHALL update an existing location via `PUT /api/locations/:id` with a JSON body.

#### Scenario: Location updated

- **WHEN** a PUT request is made to `/api/locations/1` with `{"name": "Bedroom"}`
- **AND** a location with id 1 exists
- **THEN** the API responds with HTTP 200 and the updated location JSON

#### Scenario: Location not found

- **WHEN** a PUT request is made to `/api/locations/999`
- **AND** no location with id 999 exists
- **THEN** the API responds with HTTP 404

#### Scenario: Duplicate name on update

- **WHEN** a PUT request is made to `/api/locations/1` with `{"name": "Balcony"}`
- **AND** another location named "Balcony" already exists
- **THEN** the API responds with HTTP 409 and `{"message": "..."}`

### Requirement: Delete Location

The API SHALL delete a location via `DELETE /api/locations/:id`. Plants associated with the deleted location SHALL be retained and have their `location_id` set to `null`.

#### Scenario: Location deleted

- **WHEN** a DELETE request is made to `/api/locations/1`
- **AND** a location with id 1 exists
- **THEN** the API responds with HTTP 204
- **AND** the location no longer exists
- **AND** any plants with `location_id` = 1 are retained with their `location_id` set to `null`

#### Scenario: Location not found

- **WHEN** a DELETE request is made to `/api/locations/999`
- **AND** no location with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Plant Count in List Response

The `GET /api/locations` response SHALL include a `plant_count` field for each location indicating how many plants reference that location.

#### Scenario: Location with plants

- **WHEN** a location has 3 plants assigned to it
- **THEN** the list response includes `plant_count: 3` for that location

#### Scenario: Location with no plants

- **WHEN** a location has no plants assigned to it
- **THEN** the list response includes `plant_count: 0` for that location

#### Scenario: Newly created location

- **WHEN** a location is created via `POST /api/locations`
- **THEN** the response includes `plant_count: 0`
