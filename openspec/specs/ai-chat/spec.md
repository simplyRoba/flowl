## Purpose

AI chat capability: streaming chat endpoint, semantic plant context and assistant guidance, image support, and SSE event formatting for conversational plant care assistance.

## Requirements

### Requirement: Chat endpoint

The system SHALL expose `POST /api/ai/chat` accepting a JSON body with fields `plant_id` (integer, required), `message` (string, required), `image` (optional), and `history` (array of `{ role, content, image? }` objects, optional). Current and history `image` values SHALL be base64 data URLs with media type `image/jpeg`, `image/png`, or `image/webp`. The declared media type SHALL match the encoded image bytes. The endpoint SHALL validate images before forwarding them to the AI provider. The endpoint SHALL return an SSE stream (`text/event-stream`) and enforce a maximum request body size of 30 MB.

#### Scenario: Successful chat request without image

- **WHEN** a valid JSON body with `plant_id` and `message` is sent to `POST /api/ai/chat`
- **THEN** the response SHALL have content type `text/event-stream`
- **AND** the response SHALL stream SSE events containing JSON `{"delta":"<text>"}` for each token
- **AND** the final event SHALL be `{"done":true}`

#### Scenario: Chat request with image

- **WHEN** a valid JSON body includes an `image` field containing a JPEG, PNG, or WebP base64 data URL
- **THEN** the image SHALL remain associated with the current text message and its declared media type SHALL be preserved
- **AND** the response SHALL stream as normal

#### Scenario: Chat request with conversation history

- **WHEN** a valid JSON body includes a `history` array of prior messages
- **THEN** all history messages SHALL be included as prior conversation turns in their supplied order
- **AND** the current `message` SHALL follow the history as the latest user turn

#### Scenario: Conversation history includes an image

- **WHEN** a history entry includes a valid JPEG, PNG, or WebP base64 data URL in `image`
- **THEN** that image SHALL remain associated with the corresponding prior message
- **AND** its declared media type SHALL be preserved

#### Scenario: Conversation history includes an invalid image

- **WHEN** a history entry includes an unsupported media type or invalid base64 data URL in `image`
- **THEN** the endpoint SHALL return HTTP 400 with error code `AI_INVALID_IMAGE`
- **AND** no request SHALL be sent to the AI provider

#### Scenario: AI provider not configured

- **WHEN** `POST /api/ai/chat` is called and no AI provider is configured
- **THEN** the endpoint SHALL return HTTP 503 with code `AI_NOT_CONFIGURED` and a user-safe message according to `core-api`

#### Scenario: Plant not found

- **WHEN** `POST /api/ai/chat` is called with a `plant_id` that does not exist
- **THEN** the endpoint SHALL return HTTP 404 with code `PLANT_NOT_FOUND` and a user-safe message according to `core-api`

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

### Requirement: Plant context

The system SHALL supply the AI with semantic context for the selected plant. That context SHALL include the plant's name, available species, location name, and notes; its current `watering_status` and available `last_watered` date; and its desired care preferences: `light_needs`, `watering_interval_days`, and available `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture`. Care preferences describe desired conditions for the plant, not its current state.

The supplied context SHALL also include watering dates (YYYY-MM-DD) for every watering event from the last 1 year, including watering events with notes, ordered newest first. It SHALL include records containing the event type, date, and available notes for all non-watering events and watering events with notes from the last 5 years, ordered newest first. No records older than the applicable retention period SHALL be supplied. Unavailable optional plant or event values MAY be absent or null.

#### Scenario: Plant with mixed care events

- **WHEN** context is supplied for a plant that has 40 watering events (2 with notes) and 5 fertilizing events spanning 3 years
- **THEN** it SHALL include only watering dates from the last 1 year, ordered newest first
- **AND** it SHALL include all 5 fertilizing events and the 2 watering events that have notes (if within 5 years), ordered newest first
- **AND** each watering event with notes from the last 1 year SHALL be represented both by its watering date and by its care-event details
- **AND** a watering event with notes that is older than 1 year but within 5 years SHALL appear only in the care-event details

#### Scenario: Plant with no care events

- **WHEN** context is supplied for a plant that has no care events
- **THEN** it SHALL not include any watering dates or care-event records

#### Scenario: Plant with optional fields missing

- **WHEN** the plant has `species`, `notes`, or `location_name` as `null`
- **THEN** the unavailable values SHALL be absent or null in the supplied context

#### Scenario: Care preferences describe desired conditions

- **WHEN** context is supplied for a plant with care preference fields set
- **THEN** it SHALL distinguish the current `watering_status` and `last_watered` from `light_needs`, `watering_interval_days`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and `soil_moisture` as desired care conditions

#### Scenario: Watering events older than 1 year excluded from watering dates

- **WHEN** context is supplied for a plant with watering events older than 1 year
- **THEN** it SHALL not include watering dates older than 1 year
- **AND** an older watering event with notes SHALL still be included as a care-event record if it is within 5 years

#### Scenario: Non-watering events up to 5 years included

- **WHEN** context is supplied for a plant with a repotting event from 4 years ago
- **THEN** it SHALL include that repotting event

#### Scenario: Events older than 5 years excluded

- **WHEN** context is supplied for a plant with care events older than 5 years
- **THEN** it SHALL not include those events

### Requirement: Chat rate limiting

The chat endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Chat request within rate limit

- **WHEN** a valid chat request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Chat request exceeds rate limit

- **WHEN** a chat request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider

### Requirement: Chat assistant guidance

For every chat request, the system SHALL direct the model to act as "flowl, a plant care assistant" and use the supplied plant context for personalized advice. The guidance SHALL request two to four short paragraphs, bullet points for actionable steps, acknowledgment of uncertainty, refusal of non-plant-care topics, and a response language matching the user's locale setting. It SHALL distinguish supplied care preferences as desired conditions rather than assertions about the plant's current state.

#### Scenario: Assistant uses supplied plant context

- **WHEN** a chat request is processed for a plant
- **THEN** the plant's name, available species, care preferences, and recent care events SHALL be supplied for personalized guidance

#### Scenario: Assistant distinguishes care preferences from current state

- **WHEN** a chat request is processed for a plant with care preference fields set
- **THEN** the model guidance SHALL identify those fields as desired conditions rather than the plant's current state

#### Scenario: Assistant respects locale

- **WHEN** the user's locale is `de`
- **THEN** the model SHALL be directed to respond in German

#### Scenario: Assistant restricts scope

- **WHEN** a user asks a question unrelated to plant care
- **THEN** the model SHALL have been directed to decline the unrelated question
