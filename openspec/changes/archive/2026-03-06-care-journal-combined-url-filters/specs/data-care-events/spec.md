## MODIFIED Requirements

### Requirement: List All Care Events (Global)

The API SHALL return paginated care events across all plants via `GET /api/care`, ordered by `occurred_at` descending, using cursor-based pagination. The endpoint SHALL use `axum_extra::extract::Query` (backed by `serde_qs`) to deserialize query parameters, enabling repeated `type` keys.

#### Scenario: First page of events

- **WHEN** a GET request is made to `/api/care`
- **THEN** the API responds with HTTP 200 and a JSON object containing an `events` array (up to 20 events) and a `has_more` boolean

#### Scenario: Custom page size

- **WHEN** a GET request is made to `/api/care?limit=5`
- **THEN** the API responds with at most 5 events

#### Scenario: Next page via cursor

- **GIVEN** a previous response contained an event with id 42 as the last item
- **WHEN** a GET request is made to `/api/care?before=42`
- **THEN** the API responds with events older than the event with id 42

#### Scenario: Filter by single event type

- **WHEN** a GET request is made to `/api/care?type=watered`
- **THEN** the API responds with only care events of type `watered`
- **AND** pagination and `has_more` apply to the filtered set

#### Scenario: Filter by multiple event types

- **WHEN** a GET request is made to `/api/care?type=watered&type=fertilized`
- **THEN** the API responds with only care events whose type is `watered` or `fertilized`
- **AND** pagination and `has_more` apply to the filtered set

#### Scenario: Invalid filter type in multi-type request

- **WHEN** a GET request is made to `/api/care?type=watered&type=invalid`
- **THEN** the API responds with HTTP 422

#### Scenario: No more events

- **WHEN** all events have been fetched
- **THEN** `has_more` is `false`

#### Scenario: No events exist

- **WHEN** a GET request is made to `/api/care`
- **AND** no care events exist
- **THEN** the API responds with HTTP 200, an empty `events` array, and `has_more` = `false`
