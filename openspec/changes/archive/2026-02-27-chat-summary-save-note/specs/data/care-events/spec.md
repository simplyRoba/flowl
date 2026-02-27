## MODIFIED Requirements

### Requirement: Valid Event Types

The API SHALL accept only the following event types: `watered`, `fertilized`, `repotted`, `pruned`, `custom`, `ai-consultation`.

#### Scenario: Each valid type accepted

- **WHEN** a care event is created with `event_type` set to any of `watered`, `fertilized`, `repotted`, `pruned`, `custom`, `ai-consultation`
- **THEN** the API responds with HTTP 201

#### Scenario: Invalid type rejected

- **WHEN** a care event is created with `event_type` set to `trimmed`
- **THEN** the API responds with HTTP 422

#### Scenario: AI consultation event does not trigger MQTT

- **WHEN** a care event with `event_type` = `ai-consultation` is created
- **THEN** no MQTT publish SHALL occur
