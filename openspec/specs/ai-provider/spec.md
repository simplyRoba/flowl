## Purpose

AI integration behavior: OpenAI-compatible interoperability, configuration via environment variables, identification, streamed chat, and summarization.

## Requirements

### Requirement: AI integration capabilities

When AI is enabled, the system SHALL provide identification, streamed chat, and summarization capabilities using configuration established at application startup. All AI operations SHALL use that configuration consistently until the application restarts.

#### Scenario: AI capabilities use a consistent configuration

- **WHEN** AI is enabled and an identification, chat, or summarization operation is performed
- **THEN** the operation SHALL use the configuration established at startup

### Requirement: OpenAI-compatible interoperability

The system SHALL communicate with OpenAI-compatible API endpoints. AI requests SHALL target `{base_url}/chat/completions`.

#### Scenario: Requests target configured base URL

- **WHEN** the base URL is `https://api.openai.com/v1`
- **THEN** API requests are sent to `https://api.openai.com/v1/chat/completions`

#### Scenario: Requests target custom base URL

- **WHEN** the base URL is `http://localhost:11434/v1`
- **THEN** API requests are sent to `http://localhost:11434/v1/chat/completions`

### Requirement: Streamed chat behavior

Chat requests SHALL include `stream: true`. The system SHALL process the streaming SSE response by parsing `data:` lines for delta content tokens. It SHALL ignore empty lines and the terminal `data: [DONE]` marker. The system SHALL make each parsed delta available progressively to the chat consumer.

For every current or history chat message with an image, the system SHALL represent that image as an `image_url` content part containing a base64 data URL alongside the corresponding text part. The data URL SHALL preserve the image media type and remain associated with its corresponding conversation turn.

#### Scenario: Chat streams delta tokens

- **WHEN** a chat operation receives a streaming AI response
- **THEN** each delta content token from its SSE `data:` lines SHALL be made available progressively to the chat consumer

#### Scenario: Chat handles empty lines

- **WHEN** a streaming response includes empty lines
- **THEN** the system SHALL ignore them

#### Scenario: Chat handles [DONE] marker

- **WHEN** a streaming response includes a `data: [DONE]` line
- **THEN** the system SHALL stop processing the response and complete the chat stream without error

#### Scenario: Chat includes images in their corresponding turns

- **WHEN** a chat operation includes an image in the current message or any history message
- **THEN** each image SHALL be represented by an `image_url` content part with a base64 data URL alongside that message's text part
- **AND** the image media type and association with its corresponding conversation turn SHALL be preserved

#### Scenario: Chat stream completes

- **WHEN** the streaming response ends without an error
- **THEN** the chat stream SHALL complete after its available deltas have been delivered

#### Scenario: Chat stream fails

- **WHEN** the upstream streaming response reports an error
- **THEN** the chat consumer SHALL receive an error outcome and the chat stream SHALL end

### Requirement: Structured summarization

Summarization requests SHALL be non-streaming and SHALL use `response_format: { "type": "json_schema" }` with `strict: true` and a schema requiring exactly one `summary` string field. This structured-output contract is required for provider compatibility. A conforming provider result SHALL supply the summary text; a missing or invalid structured result SHALL cause the summarization operation to fail.

#### Scenario: Summarization returns summary

- **WHEN** a summarization operation receives a conforming structured result with a summary
- **THEN** the system SHALL return the summary string

#### Scenario: Summarization handles missing or invalid summary

- **WHEN** the AI response lacks a summary or does not satisfy the required structured output
- **THEN** the summarization operation SHALL fail

### Requirement: Identification result envelope

Identification results SHALL use a JSON envelope containing `suggestions` and supporting `rejected` and `rejected_reason`. For backward compatibility, an accepted provider result MAY omit `rejected` and `rejected_reason` or set them to `null`; otherwise it SHALL have `rejected: false`. An accepted result SHALL contain one to three suggestions and no non-null rejection reason. A rejected result SHALL have `rejected: true`, zero suggestions, and a non-empty rejection reason. Newly requested structured output SHALL require all three envelope fields as defined by the identification schema.

Suggestions SHALL include `common_name` and `scientific_name`; `confidence`, `summary`, and `care_profile` MAY be absent or `null` to represent unavailable values and otherwise retain their existing meanings. `common_name` and `summary` are free-text fields, while `scientific_name` is a Latin scientific name and enum-constrained `care_profile` fields use their schema-defined English values.

#### Scenario: Identification result with multiple suggestions

- **WHEN** the AI returns `{ "suggestions": [{ "common_name": "A", "scientific_name": "B" }, { "common_name": "C", "scientific_name": "D" }], "rejected": false, "rejected_reason": null }`
- **THEN** the identification result SHALL contain the two suggestions and no rejection reason

#### Scenario: Backward-compatible accepted result

- **WHEN** the AI returns an envelope with one to three valid suggestions and omits `rejected` and `rejected_reason`
- **THEN** the identification result SHALL be accepted with no rejection reason

#### Scenario: Identification result with rejection

- **WHEN** the AI returns `{ "suggestions": [], "rejected": true, "rejected_reason": "This is a coffee mug" }`
- **THEN** the identification result SHALL be rejected with that rejection reason and no suggestions

### Requirement: Structured plant identification

The system SHALL accept one or more images and a locale for plant identification. It SHALL encode every image as a base64 data URL and include all images in one API request as separate `image_url` content parts. Each part SHALL use the OpenAI-compatible image content shape and contain its data URL. The request SHALL use structured output with `response_format: { "type": "json_schema" }`; its JSON schema SHALL define a root object with required `suggestions` (array), `rejected` (boolean), and `rejected_reason` (string or null) properties. These response-format and schema invariants are required for provider compatibility.

For a plant image, the identification request SHALL direct the model to provide one to three top likely identifications ranked by confidence, use the supplied locale for free-text `common_name` and `summary` values, retain Latin `scientific_name` values, and use the schema-defined English values for enum-constrained `care_profile` fields. For an image that does not contain a plant, the request SHALL direct the model to return `rejected: true`, a brief non-empty `rejected_reason`, and an empty `suggestions` array. For a plant image, it SHALL direct the model to return `rejected: false`, `rejected_reason: null`, and populated suggestions.

The system SHALL reject identification results that do not satisfy the result-envelope rules or cannot be interpreted as the required structured output. In accepted results, it SHALL order suggestions by descending confidence, with suggestions without confidence last.

#### Scenario: Single image identification returns multiple suggestions

- **WHEN** plant identification is requested for one image of a plant
- **THEN** the result SHALL be accepted and contain between one and three suggestions

#### Scenario: Multi-image identification returns multiple suggestions

- **WHEN** plant identification is requested for multiple images of a plant
- **THEN** all images SHALL be included in the same API request as separate `image_url` content parts
- **AND** the result SHALL be accepted and contain between one and three suggestions

#### Scenario: Suggestions are ranked by confidence

- **WHEN** the AI returns multiple suggestions
- **THEN** the suggestions SHALL be ordered by descending confidence, highest first

#### Scenario: AI returns fewer than three suggestions

- **WHEN** the AI returns one or two suggestions for an accepted result
- **THEN** the result SHALL contain only those suggestions without error

#### Scenario: AI returns incomplete optional fields

- **WHEN** the AI response omits optional `confidence`, `summary`, or `care_profile` fields or sets them to `null`
- **THEN** the identification result SHALL treat those fields as unavailable

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be interpreted as the required identification result envelope
- **THEN** the identification operation SHALL fail

#### Scenario: AI returns inconsistent accepted results

- **WHEN** an accepted response contains zero or more than three suggestions or a rejection reason
- **THEN** the identification operation SHALL fail

#### Scenario: AI returns inconsistent rejected results

- **WHEN** a rejected response contains suggestions or lacks a non-empty rejection reason
- **THEN** the identification operation SHALL fail

#### Scenario: Accepted suggestions are ordered by confidence

- **WHEN** an accepted response contains suggestions in a different order
- **THEN** the system SHALL return them in descending confidence order with missing confidence values last

#### Scenario: JSON schema wraps results in suggestions array

- **WHEN** the identification request is prepared
- **THEN** the `json_schema` response format SHALL define a root object with required `suggestions` (array), `rejected` (boolean), and `rejected_reason` (string or null) properties

#### Scenario: Non-plant photo triggers rejection

- **WHEN** plant identification is requested for an image that does not contain a plant
- **THEN** the result SHALL be rejected with a non-empty `rejected_reason` and an empty `suggestions` array

### Requirement: AI configuration via environment variables

The system SHALL read AI configuration from environment variables: `FLOWL_AI_API_KEY` (required to enable AI, no default), `FLOWL_AI_BASE_URL` (default: `https://api.openai.com/v1`), and `FLOWL_AI_MODEL` (default: `gpt-4.1-mini`).

#### Scenario: All AI environment variables set

- **WHEN** `FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, and `FLOWL_AI_MODEL` are set at startup
- **THEN** AI operations SHALL use the specified values

#### Scenario: Only API key set

- **WHEN** only `FLOWL_AI_API_KEY` is set at startup
- **THEN** AI operations SHALL use base URL `https://api.openai.com/v1` and model `gpt-4.1-mini`

#### Scenario: No API key set

- **WHEN** `FLOWL_AI_API_KEY` is not set at startup
- **THEN** AI capabilities SHALL be unavailable

### Requirement: AI availability at startup

The system SHALL determine AI availability at startup: AI SHALL be available when `FLOWL_AI_API_KEY` is set and unavailable otherwise. That availability and the associated configuration SHALL remain in effect until the application restarts.

#### Scenario: AI enabled at startup

- **WHEN** the application starts with `FLOWL_AI_API_KEY` set
- **THEN** AI capabilities SHALL be available

#### Scenario: AI disabled at startup

- **WHEN** the application starts without `FLOWL_AI_API_KEY`
- **THEN** AI capabilities SHALL be unavailable
