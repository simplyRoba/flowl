## Purpose

AI summarization capability: conversation-to-journal-note condensation endpoint, JSON-mode output, and locale-aware summaries for plant care conversations.

## Requirements

### Requirement: Summarize endpoint

The system SHALL expose `POST /api/ai/summarize` accepting a JSON body with fields `plant_id` (integer, required) and `history` (array of `{ role, content }` objects, required). The endpoint SHALL return a JSON response `{"summary":"<text>"}` containing a 1–3 sentence summary of the conversation suitable for a care journal entry.

#### Scenario: Successful summarization

- **WHEN** a valid JSON body with `plant_id` and `history` (at least one exchange) is sent to `POST /api/ai/summarize`
- **THEN** the response SHALL be HTTP 200 with `{"summary":"<1-3 sentence summary>"}`

#### Scenario: AI provider not configured

- **WHEN** `POST /api/ai/summarize` is called and no AI provider is configured
- **THEN** the endpoint SHALL return HTTP 503 with `{"message":"AI provider is not configured"}`

#### Scenario: Plant not found

- **WHEN** `POST /api/ai/summarize` is called with a `plant_id` that does not exist
- **THEN** the endpoint SHALL return HTTP 404

#### Scenario: Empty history

- **WHEN** `POST /api/ai/summarize` is called with an empty `history` array
- **THEN** the endpoint SHALL return HTTP 422 with a validation error

### Requirement: Summarize uses JSON mode

The `summarize` provider method SHALL use `response_format: { "type": "json_object" }` to guarantee valid JSON output. The system prompt SHALL instruct the model to return `{"summary":"..."}`. The response SHALL be deserialized and the `summary` field extracted.

#### Scenario: Valid JSON response

- **WHEN** the AI returns `{"summary":"Diagnosed yellowing as overwatering."}`
- **THEN** the endpoint SHALL return that summary string in the response

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be parsed as JSON or lacks a `summary` field
- **THEN** the endpoint SHALL return HTTP 500 with an internal error message

### Requirement: Summarize system prompt

The system prompt for summarization SHALL instruct the model to condense the conversation into a 1–3 sentence care journal note. It SHALL include the plant's name and species for context. It SHALL instruct the model to focus on diagnoses, advice given, and actions recommended. The response language SHALL match the user's locale setting.

#### Scenario: Summary reflects conversation content

- **WHEN** a conversation about yellowing leaves and overwatering is summarized
- **THEN** the summary SHALL mention the diagnosis and recommendation

#### Scenario: Summary respects locale

- **WHEN** the user's locale is `es`
- **THEN** the summary SHALL be written in Spanish
