## Purpose

AI summarization capability: conversation-to-journal-note condensation endpoint and locale-aware summaries for plant care conversations.

## Requirements

### Requirement: Summarize endpoint

The system SHALL expose `POST /api/ai/summarize` accepting a JSON body with fields `plant_id` (integer, required) and `history` (array of `{ role, content }` objects, required). The endpoint SHALL return a JSON response `{"summary":"<text>"}` containing a 1–3 sentence summary of the conversation suitable for a care journal entry.

#### Scenario: Successful summarization

- **WHEN** a valid JSON body with `plant_id` and `history` (at least one exchange) is sent to `POST /api/ai/summarize`
- **THEN** the response SHALL be HTTP 200 with `{"summary":"<1-3 sentence summary>"}`

#### Scenario: AI provider not configured

- **WHEN** `POST /api/ai/summarize` is called and no AI provider is configured
- **THEN** the endpoint SHALL return HTTP 503 with code `AI_NOT_CONFIGURED` and a user-safe message according to `core-api`

#### Scenario: Plant not found

- **WHEN** `POST /api/ai/summarize` is called with a `plant_id` that does not exist
- **THEN** the endpoint SHALL return HTTP 404 with code `PLANT_NOT_FOUND` and a user-safe message according to `core-api`

#### Scenario: Empty history

- **WHEN** `POST /api/ai/summarize` is called with an empty `history` array
- **THEN** the endpoint SHALL return HTTP 422 with code `AI_HISTORY_EMPTY` and a user-safe message according to `core-api`

### Requirement: Summarize uses structured output

The summarize endpoint SHALL rely on the structured-output interoperability defined by `ai-provider`. A conforming provider result SHALL supply a valid summary for the endpoint response. A malformed or missing summary SHALL cause the endpoint to return its existing HTTP 500 safe internal error.

#### Scenario: Valid structured result

- **WHEN** the AI returns a valid structured result with summary `Diagnosed yellowing as overwatering.`
- **THEN** the endpoint SHALL return that summary string in the response

#### Scenario: AI returns malformed or missing summary

- **WHEN** the AI result is malformed or does not supply a summary
- **THEN** the endpoint SHALL return HTTP 500 with its safe internal error message

### Requirement: Summarize rate limiting

The summarize endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Summarize request within rate limit

- **WHEN** a valid summarize request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Summarize request exceeds rate limit

- **WHEN** a summarize request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider

### Requirement: Summary generation

A generated summary SHALL be a one-to-three-sentence care journal note that uses the plant's name and available species as context. It SHALL focus on diagnoses, advice given, and actions recommended in the conversation. The summary language SHALL match the user's locale setting.

#### Scenario: Summary reflects conversation content

- **WHEN** a conversation about yellowing leaves and overwatering is summarized
- **THEN** the summary SHALL mention the diagnosis and recommendation

#### Scenario: Summary respects locale

- **WHEN** the user's locale is `es`
- **THEN** the summary SHALL be written in Spanish
