## MODIFIED Requirements

### Requirement: Plant context builder

The system SHALL build a plant context for the AI system prompt by loading the plant record and the 20 most recent care events from the database. The context SHALL be serialized as a JSON object with the following top-level structure:

- `name` (string)
- `species` (string, optional)
- `location_name` (string, optional)
- `notes` (string, optional)
- `current_state` — an object containing fields that describe the plant's current condition: `watering_status` (string) and `last_watered` (string, optional)
- `care_preferences` — an object containing fields that describe the desired care profile: `light_needs` (string), `watering_interval_days` (integer), `difficulty` (string, optional), `pet_safety` (string, optional), `growth_speed` (string, optional), `soil_type` (string, optional), `soil_moisture` (string, optional)
- `recent_care_events` — an array of objects each with `event_type` (string), `date` (string), and optional `notes` (string)

#### Scenario: Plant with care events

- **WHEN** context is built for a plant that has 30 care events
- **THEN** the context SHALL include the plant fields and the 20 most recent care events ordered by date descending

#### Scenario: Plant with no care events

- **WHEN** context is built for a plant that has no care events
- **THEN** the context SHALL include the plant fields and an empty `recent_care_events` array

#### Scenario: Plant with optional fields missing

- **WHEN** the plant has `species`, `notes`, or `location_name` as NULL
- **THEN** those fields SHALL be omitted or null in the context JSON

#### Scenario: Care preferences grouped separately from current state

- **WHEN** the context is serialized to JSON
- **THEN** `watering_status` and `last_watered` SHALL appear under the `current_state` object
- **AND** `light_needs`, `watering_interval_days`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture` SHALL appear under the `care_preferences` object
- **AND** care preference fields SHALL NOT appear at the top level

### Requirement: Chat system prompt

The system SHALL prepend a system message to every chat request. The system prompt SHALL establish the assistant identity as "flowl, a plant care assistant", instruct it to use the provided plant context for personalized advice, set a concise response style (2-4 short paragraphs, bullet points for actionable steps), instruct it to acknowledge uncertainty, and restrict responses to plant-care topics. The system prompt SHALL include the serialized plant context JSON. The system prompt SHALL explicitly instruct the model that the `care_preferences` section describes desired conditions for the plant, not its current state. The system prompt SHALL instruct the model to respond in the language matching the user's locale setting.

#### Scenario: System prompt includes plant context

- **WHEN** a chat request is processed for a plant
- **THEN** the AI request SHALL include a system message containing the plant's name, species, care profile, and recent care events

#### Scenario: System prompt clarifies care preferences

- **WHEN** a chat request is processed for a plant with care preference fields set
- **THEN** the system prompt SHALL contain an instruction clarifying that the `care_preferences` section describes the desired conditions, not the current state of the plant

#### Scenario: System prompt sets locale

- **WHEN** the user's locale is `de`
- **THEN** the system prompt SHALL instruct the model to respond in German

#### Scenario: System prompt restricts scope

- **WHEN** a user asks a question unrelated to plant care
- **THEN** the system prompt SHALL have instructed the model to decline non-plant-care questions
