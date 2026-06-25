## MODIFIED Requirements

### Requirement: Create Care Event

The API SHALL create a care event via `POST /api/plants/:id/care` with a JSON body.

#### Scenario: Valid care event created

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "fertilized", "notes": "Used liquid fertilizer"}`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 201 and the created care event JSON
- **AND** `occurred_at` defaults to the current datetime
- **AND** `plant_id` is set to 1

#### Scenario: Care event with explicit occurred_at

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "repotted", "occurred_at": "2026-02-14T10:00:00"}`
- **AND** a plant with id 1 exists
- **THEN** the API responds with HTTP 201
- **AND** `occurred_at` is set to the provided value

#### Scenario: Watered event triggers MQTT publish

- **WHEN** a care event with `event_type` = `watered` is created for a plant
- **THEN** the plant's watering state and attributes SHALL be published to MQTT

#### Scenario: Non-watered event skips MQTT publish

- **WHEN** a care event with `event_type` other than `watered` is created
- **THEN** no MQTT publish SHALL occur

#### Scenario: Invalid event type

- **WHEN** a POST request is made to `/api/plants/1/care` with `{"event_type": "unknown"}`
- **THEN** the API responds with HTTP 422
- **AND** the error message indicates the valid event types

#### Scenario: Event type missing

- **WHEN** a POST request is made to `/api/plants/1/care` with `{}`
- **THEN** the API responds with HTTP 422

#### Scenario: Plant not found

- **WHEN** a POST request is made to `/api/plants/999/care` with `{"event_type": "watered"}`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

### Requirement: Delete Care Event

The API SHALL delete a care event via `DELETE /api/plants/:id/care/:event_id`.

#### Scenario: Care event deleted

- **GIVEN** a care event with id 5 belongs to plant with id 1
- **WHEN** a DELETE request is made to `/api/plants/1/care/5`
- **THEN** the API responds with HTTP 204
- **AND** the care event is removed from the database

#### Scenario: Watered event deletion triggers MQTT publish

- **WHEN** a care event with `event_type` = `watered` is deleted
- **THEN** the plant's watering state and attributes SHALL be republished to MQTT with the updated `last_watered` derived from remaining care events

#### Scenario: Non-watered event deletion skips MQTT publish

- **WHEN** a care event with `event_type` other than `watered` is deleted
- **THEN** no MQTT publish SHALL occur

#### Scenario: Care event not found

- **WHEN** a DELETE request is made to `/api/plants/1/care/999`
- **AND** no care event with id 999 exists for plant 1
- **THEN** the API responds with HTTP 404

#### Scenario: Plant not found

- **WHEN** a DELETE request is made to `/api/plants/999/care/1`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404
