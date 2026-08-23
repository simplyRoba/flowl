## MODIFIED Requirements

### Requirement: List All Care Events (Global)

The API SHALL return bounded, paginated care events across all plants via `GET /api/care`, ordered by `occurred_at` descending and then `id` descending. Cursor pagination SHALL use the cursor event's `(occurred_at, id)` position in that same ordering. The endpoint SHALL deserialize repeated `type` query keys as multiple event-type filters.

#### Scenario: First page of events

- **WHEN** a GET request is made to `/api/care`
- **THEN** the API responds with HTTP 200 and a JSON object containing an `events` array with up to 20 events and a `has_more` boolean

#### Scenario: Custom page size

- **WHEN** a GET request is made to `/api/care?limit=5`
- **THEN** the API responds with at most 5 events

#### Scenario: Journal page size

- **WHEN** a GET request is made to `/api/care?limit=500`
- **THEN** the API responds with at most 500 events
- **AND** `has_more` indicates whether another matching event exists

#### Scenario: Page size remains bounded

- **WHEN** a GET request asks for more than 500 events
- **THEN** the API responds with at most 500 events

#### Scenario: Next page via cursor

- **GIVEN** a previous response contained event 42 as its last item
- **WHEN** a GET request is made to `/api/care?before=42`
- **THEN** the API responds only with events whose `(occurred_at, id)` positions sort after event 42 in the descending journal order
- **AND** event 42 is not repeated

#### Scenario: Backdated event pagination

- **GIVEN** event IDs are not in the same order as their `occurred_at` timestamps
- **WHEN** all pages are requested by passing each page's last event ID as `before`
- **THEN** every matching event is returned exactly once in `(occurred_at DESC, id DESC)` order

#### Scenario: Equal timestamp pagination

- **GIVEN** multiple events have the same `occurred_at` timestamp
- **WHEN** a page boundary falls between those events
- **THEN** descending `id` order determines their position
- **AND** every event at that timestamp is returned exactly once across pages

#### Scenario: Supported timestamp representations

- **GIVEN** care events use valid UTC, explicit-offset, or legacy SQLite-compatible timestamps
- **WHEN** the global journal is requested across multiple pages
- **THEN** the events SHALL be ordered by their actual chronological instants rather than raw timestamp text
- **AND** every event SHALL be returned exactly once

#### Scenario: Malformed historical timestamp fallback

- **GIVEN** malformed timestamps exist in historical or imported care events
- **WHEN** the global journal is requested across multiple pages
- **THEN** valid timestamps SHALL sort before malformed timestamps
- **AND** malformed events SHALL use descending `id` as a deterministic fallback order
- **AND** every event SHALL remain reachable exactly once

#### Scenario: Unknown cursor event

- **WHEN** a GET request supplies a `before` ID that does not identify a care event
- **THEN** the API responds with HTTP 422 and a JSON error message

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

- **WHEN** all matching events have been fetched
- **THEN** `has_more` is `false`

#### Scenario: No events exist

- **WHEN** a GET request is made to `/api/care`
- **AND** no care events exist
- **THEN** the API responds with HTTP 200, an empty `events` array, and `has_more` equal to `false`

## ADDED Requirements

### Requirement: Care Event Occurrence Timestamp Validation

The API SHALL reject malformed explicit occurrence timestamps before creating care events so chronological journal ordering remains well-defined.

#### Scenario: Invalid occurrence timestamp on create

- **WHEN** a POST request to `/api/plants/:id/care` supplies an `occurred_at` value that is not a supported datetime
- **THEN** the API responds with HTTP 422 and a JSON error message
- **AND** no care event is created
