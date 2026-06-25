## MODIFIED Requirements

### Requirement: Plant context builder

The system SHALL build a plant context for the AI system prompt by loading the plant record, watering dates from the last 1 year, and all non-watering care events (plus watering events with notes) from the last 5 years. The context SHALL be serialized as a JSON object with the following top-level structure:

- `name` (string)
- `species` (string, optional)
- `location_name` (string, optional)
- `notes` (string, optional)
- `current_state` — an object containing fields that describe the plant's current condition: `watering_status` (string) and `last_watered` (string, optional)
- `care_preferences` — an object containing fields that describe the desired care profile: `light_needs` (string), `watering_interval_days` (integer), `difficulty` (string, optional), `pet_safety` (string, optional), `growth_speed` (string, optional), `soil_type` (string, optional), `soil_moisture` (string, optional)
- `watering_dates` — an array of date strings (YYYY-MM-DD) for all watering events from the last 1 year, ordered most recent first. Watering events with notes SHALL be included in this list.
- `care_events` — an array of objects each with `event_type` (string), `date` (string), and optional `notes` (string), containing all non-watering events from the last 5 years plus watering events that have notes, ordered most recent first.

Optional collection fields (`watering_dates`, `care_events`) SHALL be omitted from the JSON when empty.

#### Scenario: Plant with mixed care events

- **WHEN** context is built for a plant that has 40 watering events (2 with notes) and 5 fertilizing events spanning 3 years
- **THEN** the `watering_dates` array SHALL contain only watering dates from the last 1 year, ordered most recent first
- **AND** the `care_events` array SHALL contain all 5 fertilizing events and the 2 watering events that have notes (if within 5 years), ordered most recent first
- **AND** the 2 watering events with notes SHALL appear in both `watering_dates` (as dates) and `care_events` (as full objects)

#### Scenario: Plant with no care events

- **WHEN** context is built for a plant that has no care events
- **THEN** `watering_dates` and `care_events` SHALL be omitted from the context JSON

#### Scenario: Plant with optional fields missing

- **WHEN** the plant has `species`, `notes`, or `location_name` as NULL
- **THEN** those fields SHALL be omitted or null in the context JSON

#### Scenario: Care preferences grouped separately from current state

- **WHEN** the context is serialized to JSON
- **THEN** `watering_status` and `last_watered` SHALL appear under the `current_state` object
- **AND** `light_needs`, `watering_interval_days`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture` SHALL appear under the `care_preferences` object
- **AND** care preference fields SHALL NOT appear at the top level

#### Scenario: Watering events older than 1 year excluded from watering_dates

- **WHEN** context is built for a plant with watering events older than 1 year
- **THEN** `watering_dates` SHALL NOT contain dates older than 1 year
- **AND** watering events older than 1 year that have notes SHALL still appear in `care_events` if within 5 years

#### Scenario: Non-watering events up to 5 years included

- **WHEN** context is built for a plant with a repotting event from 4 years ago
- **THEN** the repotting event SHALL appear in `care_events`

#### Scenario: Events older than 5 years excluded

- **WHEN** context is built for a plant with care events older than 5 years
- **THEN** no events older than 5 years SHALL appear in either `watering_dates` or `care_events`
