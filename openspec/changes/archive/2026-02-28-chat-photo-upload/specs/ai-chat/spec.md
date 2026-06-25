## MODIFIED Requirements

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
- **THEN** the endpoint SHALL send an SSE event `{"error":"<message>"}` and close the stream
