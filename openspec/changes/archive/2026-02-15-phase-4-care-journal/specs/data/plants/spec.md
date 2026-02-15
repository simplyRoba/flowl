## MODIFIED Requirements

### Requirement: Water Plant

The API SHALL record a watering event via `POST /api/plants/:id/water`.

#### Scenario: Plant watered successfully

- **WHEN** a POST request is made to `/api/plants/1/water`
- **AND** a plant with id 1 exists
- **THEN** the plant's `last_watered` is set to the current datetime
- **AND** `updated_at` is refreshed
- **AND** the API responds with HTTP 200 and the updated plant JSON with recomputed `watering_status`
- **AND** a care event with `event_type` = `watered` and `occurred_at` = current datetime is automatically created

#### Scenario: Plant not found

- **WHEN** a POST request is made to `/api/plants/999/water`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404
