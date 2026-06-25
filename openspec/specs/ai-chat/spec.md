## Purpose

AI chat capability: streaming chat endpoint, plant context assembly, system prompt construction, image support, and SSE event formatting for conversational plant care assistance.

## Requirements

### Requirement: Chat endpoint

The system SHALL expose `POST /api/ai/chat` accepting a JSON body with fields `plant_id` (integer, required), `message` (string, required), `image` (base64-encoded string, optional), and `history` (array of `{ role, content }` objects, optional). The `history` objects SHALL NOT include image data. The endpoint SHALL return an SSE stream (`text/event-stream`). A `DefaultBodyLimit` of 30 MB SHALL be applied to the route.

#### Scenario: Successful chat request without image

- **WHEN** a valid JSON body with `plant_id` and `message` is sent to `POST /api/ai/chat`
- **THEN** the response SHALL have content type `text/event-stream`
- **AND** the response SHALL stream SSE events containing JSON `{"delta":"<text>"}` for each token
- **AND** the final event SHALL be `{"done":true}`

#### Scenario: Chat request with image

- **WHEN** a valid JSON body includes an `image` field containing a base64-encoded JPEG/PNG/WebP string
- **THEN** the image SHALL be decoded and sent to the AI provider alongside the text message
- **AND** the response SHALL stream as normal

#### Scenario: Chat request with conversation history

- **WHEN** a valid JSON body includes a `history` array of prior messages
- **THEN** all history messages SHALL be included in the AI request as prior conversation turns
- **AND** the current `message` SHALL be appended as the latest user turn
- **AND** image data SHALL NOT be included in history entries

#### Scenario: AI provider not configured

- **WHEN** `POST /api/ai/chat` is called and no AI provider is configured
- **THEN** the endpoint SHALL return HTTP 503 with `{"message":"AI provider is not configured"}`

#### Scenario: Plant not found

- **WHEN** `POST /api/ai/chat` is called with a `plant_id` that does not exist
- **THEN** the endpoint SHALL return HTTP 404

#### Scenario: Mid-stream error

- **WHEN** the AI provider encounters an error while streaming
- **THEN** the endpoint SHALL send an SSE event `{"error":{"code":"<CODE>","message":"<message>"}}` and close the stream

### Requirement: SSE event format

The chat endpoint SHALL emit SSE events as JSON objects. Each event MUST be one of: `{"delta":"<text chunk>"}` for content tokens, `{"done":true}` for stream completion, or `{"error":{"code":"<CODE>","message":"<message>"}}` for errors. The error object follows the same structured `code` + `message` pattern used by REST API error responses. No other event shapes SHALL be emitted.

#### Scenario: Delta event

- **WHEN** the AI provider yields a text token
- **THEN** an SSE event `data: {"delta":"<token>"}` SHALL be sent

#### Scenario: Done event

- **WHEN** the AI provider finishes generating
- **THEN** an SSE event `data: {"done":true}` SHALL be sent
- **AND** the stream SHALL close

#### Scenario: Error event

- **WHEN** an error occurs during streaming
- **THEN** an SSE event `data: {"error":{"code":"<CODE>","message":"<message>"}}` SHALL be sent
- **AND** the stream SHALL close

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

### Requirement: Chat rate limiting

The chat endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Chat request within rate limit

- **WHEN** a valid chat request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Chat request exceeds rate limit

- **WHEN** a chat request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider

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
