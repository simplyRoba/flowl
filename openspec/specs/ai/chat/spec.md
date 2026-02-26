## Purpose

AI chat capability: streaming chat endpoint, plant context assembly, system prompt construction, image support, and SSE event formatting for conversational plant care assistance.

## Requirements

### Requirement: Chat endpoint

The system SHALL expose `POST /api/ai/chat` accepting a JSON body with fields `plant_id` (integer, required), `message` (string, required), `image` (base64-encoded string, optional), and `history` (array of `{ role, content }` objects, optional). The endpoint SHALL return an SSE stream (`text/event-stream`). A `DefaultBodyLimit` of 30 MB SHALL be applied to the route.

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

#### Scenario: AI provider not configured

- **WHEN** `POST /api/ai/chat` is called and no AI provider is configured
- **THEN** the endpoint SHALL return HTTP 503 with `{"message":"AI provider is not configured"}`

#### Scenario: Plant not found

- **WHEN** `POST /api/ai/chat` is called with a `plant_id` that does not exist
- **THEN** the endpoint SHALL return HTTP 404

#### Scenario: Mid-stream error

- **WHEN** the AI provider encounters an error while streaming
- **THEN** the endpoint SHALL send an SSE event `{"error":"<message>"}` and close the stream

### Requirement: SSE event format

The chat endpoint SHALL emit SSE events as JSON objects. Each event MUST be one of: `{"delta":"<text chunk>"}` for content tokens, `{"done":true}` for stream completion, or `{"error":"<message>"}` for errors. No other event shapes SHALL be emitted.

#### Scenario: Delta event

- **WHEN** the AI provider yields a text token
- **THEN** an SSE event `data: {"delta":"<token>"}` SHALL be sent

#### Scenario: Done event

- **WHEN** the AI provider finishes generating
- **THEN** an SSE event `data: {"done":true}` SHALL be sent
- **AND** the stream SHALL close

#### Scenario: Error event

- **WHEN** an error occurs during streaming
- **THEN** an SSE event `data: {"error":"<message>"}` SHALL be sent
- **AND** the stream SHALL close

### Requirement: Plant context builder

The system SHALL build a plant context for the AI system prompt by loading the plant record and the 20 most recent care events from the database. The context SHALL be serialized as a JSON object containing plant fields (`name`, `species`, `location_name`, `light_needs`, `watering_interval_days`, `watering_status`, `last_watered`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture`, `notes`) and a `recent_care_events` array (each with `event_type`, `date`, and optional `notes`).

#### Scenario: Plant with care events

- **WHEN** context is built for a plant that has 30 care events
- **THEN** the context SHALL include the plant fields and the 20 most recent care events ordered by date descending

#### Scenario: Plant with no care events

- **WHEN** context is built for a plant that has no care events
- **THEN** the context SHALL include the plant fields and an empty `recent_care_events` array

#### Scenario: Plant with optional fields missing

- **WHEN** the plant has `species`, `notes`, or `location_name` as NULL
- **THEN** those fields SHALL be omitted or null in the context JSON

### Requirement: Chat system prompt

The system SHALL prepend a system message to every chat request. The system prompt SHALL establish the assistant identity as "flowl, a plant care assistant", instruct it to use the provided plant context for personalized advice, set a concise response style (2–4 short paragraphs, bullet points for actionable steps), instruct it to acknowledge uncertainty, and restrict responses to plant-care topics. The system prompt SHALL include the serialized plant context JSON. The system prompt SHALL instruct the model to respond in the language matching the user's locale setting.

#### Scenario: System prompt includes plant context

- **WHEN** a chat request is processed for a plant
- **THEN** the AI request SHALL include a system message containing the plant's name, species, care profile, and recent care events

#### Scenario: System prompt sets locale

- **WHEN** the user's locale is `de`
- **THEN** the system prompt SHALL instruct the model to respond in German

#### Scenario: System prompt restricts scope

- **WHEN** a user asks a question unrelated to plant care
- **THEN** the system prompt SHALL have instructed the model to decline non-plant-care questions
