## ADDED Requirements

### Requirement: Update Care Event

The API SHALL update an existing care event via `PUT /api/plants/:id/care/:event_id` using a JSON body containing `event_type`, `notes`, and `occurred_at`. The update SHALL preserve the event's `id`, `plant_id`, `photo_path`, and `created_at`, and SHALL return the updated care event in the standard response format.

#### Scenario: Care event updated

- **GIVEN** care event 5 belongs to plant 1
- **WHEN** a PUT request is made to `/api/plants/1/care/5` with valid event type, notes, and occurrence time
- **THEN** the API responds with HTTP 200 and the updated care event JSON
- **AND** the stored editable fields match the request
- **AND** the event identity, plant ownership, existing photo, and creation timestamp are unchanged

#### Scenario: Nullable notes cleared

- **GIVEN** care event 5 has notes
- **WHEN** it is updated with `notes` set to `null`
- **THEN** the API responds with HTTP 200
- **AND** the event's notes are cleared

#### Scenario: Invalid update input

- **WHEN** a care-event update omits a required editable field, supplies an unsupported event type, or supplies an invalid occurrence time
- **THEN** the API responds with HTTP 422 and a user-safe validation message
- **AND** the existing event remains unchanged

#### Scenario: Care event does not belong to plant

- **GIVEN** care event 5 belongs to plant 2
- **WHEN** a PUT request is made to `/api/plants/1/care/5`
- **THEN** the API responds with HTTP 404
- **AND** care event 5 remains unchanged

#### Scenario: Plant not found

- **WHEN** a PUT request is made to `/api/plants/999/care/5`
- **AND** no plant with id 999 exists
- **THEN** the API responds with HTTP 404

#### Scenario: Watering history affected by update

- **GIVEN** an update changes an event from `watered`, to `watered`, or changes the occurrence time of a `watered` event
- **WHEN** the update succeeds
- **THEN** the plant's watering state and attributes SHALL be republished to MQTT using the updated care history

#### Scenario: Watering history unaffected by update

- **GIVEN** both the existing and updated event types are not `watered`
- **WHEN** the update succeeds
- **THEN** no watering-state MQTT publish SHALL occur
