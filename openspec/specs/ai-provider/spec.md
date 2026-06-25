## Purpose

AI provider abstraction layer: trait definition, OpenAI-compatible client implementation, configuration via environment variables, and provider lifecycle in AppState.

## Requirements

### Requirement: AI provider trait

The system SHALL define an `AiProvider` async trait with methods `identify`, `chat`, and `summarize`. The trait MUST be object-safe and `Send + Sync` so it can be shared across async request handlers via `Arc<dyn AiProvider>`.

The `chat` method SHALL accept a system prompt string, a slice of `ChatMessage`, an optional image as a byte slice, and a locale string. It SHALL return a `ChatResponseStream` (`ReceiverStream<Result<String, String>>`).

The `summarize` method SHALL accept a system prompt string, a slice of `ChatMessage`, and a locale string. It SHALL return a `Result<String>` containing the summary text.

#### Scenario: Trait is object-safe

- **WHEN** an `AiProvider` implementation is constructed
- **THEN** it can be stored as `Arc<dyn AiProvider>` in shared application state

#### Scenario: Chat method signature

- **WHEN** `chat` is called with a system prompt, messages, optional image bytes, and locale
- **THEN** it SHALL return a `ChatResponseStream` that yields text deltas

#### Scenario: Summarize method signature

- **WHEN** `summarize` is called with a system prompt, messages, and locale
- **THEN** it SHALL return a `Result<String>` containing the summary

### Requirement: OpenAI-compatible provider

The system SHALL implement `OpenAiProvider` that communicates with any OpenAI-compatible API endpoint. The provider MUST use a single `reqwest::Client` instance for connection pooling and MUST send requests to `{base_url}/chat/completions`.

The `chat` implementation SHALL send requests with `stream: true`, read the SSE response via `reqwest`'s streaming support, parse `data:` lines to extract delta content tokens, and forward them through an `mpsc` channel. It SHALL skip empty lines and `data: [DONE]` markers. It SHALL handle the optional image by encoding it as a base64 data URL in the latest user message's content array.

The `summarize` implementation SHALL send a non-streaming request with `response_format: { "type": "json_schema" }` using `strict: true` and a schema requiring a single `summary` string field, deserialize the response, and extract the `summary` field.

#### Scenario: Provider targets configured base URL

- **WHEN** `OpenAiProvider` is constructed with base URL `https://api.openai.com/v1`
- **THEN** API requests are sent to `https://api.openai.com/v1/chat/completions`

#### Scenario: Provider targets custom base URL

- **WHEN** `OpenAiProvider` is constructed with base URL `http://localhost:11434/v1`
- **THEN** API requests are sent to `http://localhost:11434/v1/chat/completions`

#### Scenario: Chat streams tokens via channel

- **WHEN** `chat` is called and the AI returns a streaming response
- **THEN** the provider SHALL spawn a background task that parses SSE `data:` lines
- **AND** each delta content token SHALL be sent through the `mpsc` channel
- **AND** the channel SHALL be closed when the stream ends or on error

#### Scenario: Chat handles [DONE] marker

- **WHEN** the streaming response includes a `data: [DONE]` line
- **THEN** the provider SHALL stop processing and close the channel without error

#### Scenario: Chat includes image in request

- **WHEN** `chat` is called with `image: Some(bytes)`
- **THEN** the latest user message content SHALL be an array containing a text part and an `image_url` part with the base64-encoded data URL

#### Scenario: Summarize returns extracted summary

- **WHEN** `summarize` is called and the AI returns `{"summary":"..."}`
- **THEN** the provider SHALL return the summary string

#### Scenario: Summarize handles missing summary field

- **WHEN** the AI returns valid JSON without a `summary` field
- **THEN** the provider SHALL return an error

### Requirement: IdentifyResponse wrapper type

The system SHALL define an `IdentifyResponse` struct containing a `suggestions` field of type `Vec<IdentifyResult>`, a `rejected` field of type `Option<bool>`, and a `rejected_reason` field of type `Option<String>`. The struct SHALL derive `Serialize` and `Deserialize`. When `rejected` is `true`, `suggestions` SHALL be empty and `rejected_reason` SHALL contain the AI's explanation. When `rejected` is `false` or `None`, `suggestions` SHALL contain 1–3 results and `rejected_reason` SHALL be `None`.

#### Scenario: IdentifyResponse with multiple suggestions

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [{ "common_name": "A", "scientific_name": "B" }, { "common_name": "C", "scientific_name": "D" }], "rejected": false, "rejected_reason": null }`
- **THEN** the `suggestions` field SHALL contain 2 `IdentifyResult` entries
- **AND** `rejected` SHALL be `Some(false)`

#### Scenario: IdentifyResponse serialization round-trip

- **WHEN** an `IdentifyResponse` is serialized to JSON
- **THEN** the output SHALL contain a `suggestions` array with each suggestion's fields

#### Scenario: IdentifyResponse with rejection

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [], "rejected": true, "rejected_reason": "This is a coffee mug" }`
- **THEN** `rejected` SHALL be `Some(true)`
- **AND** `rejected_reason` SHALL be `Some("This is a coffee mug")`
- **AND** `suggestions` SHALL be empty

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices) and a `locale` string, encode the images as base64 data URLs, send them to the configured model using structured output (`response_format: { "type": "json_schema" }`) with the `IdentifyResponse` schema, and deserialize the response into an `IdentifyResponse` containing a `suggestions` array of up to 3 `IdentifyResult` entries ranked by confidence. The prompt SHALL instruct the model to provide its top 3 most likely identifications. The prompt SHALL instruct the model to respond in the language matching the given locale for free-text fields (`common_name`, `summary`) while keeping `scientific_name` in Latin. Enum-constrained fields in `care_profile` remain in English by virtue of the JSON schema constraints. The `IdentifyResult` and `CareProfile` types SHALL derive both `Serialize` and `Deserialize` so they can be used as HTTP response bodies.

The JSON schema sent to the AI SHALL include top-level `rejected` (boolean, required) and `rejected_reason` (string or null, required) fields alongside the existing `suggestions` array. The prompt SHALL instruct the model to set `rejected` to `true` with a brief `rejected_reason` and an empty `suggestions` array when the photo does not contain a plant. When the photo does contain a plant, the model SHALL set `rejected` to `false`, `rejected_reason` to `null`, and populate `suggestions` as before.

#### Scenario: Single image identification returns multiple suggestions

- **WHEN** `identify` is called with one image of a plant
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with `rejected: false` and up to 3 suggestions

#### Scenario: Multi-image identification returns multiple suggestions

- **WHEN** `identify` is called with multiple images of a plant
- **THEN** all images are included in the same API request as separate image content parts
- **AND** the response SHALL contain `rejected: false` and up to 3 suggestions

#### Scenario: Suggestions are ranked by confidence

- **WHEN** the AI returns multiple suggestions
- **THEN** the suggestions SHALL be ordered by descending confidence (highest first)

#### Scenario: AI returns fewer than 3 suggestions

- **WHEN** the AI returns only 1 or 2 suggestions
- **THEN** the `IdentifyResponse` SHALL contain only the returned suggestions without error

#### Scenario: AI returns incomplete optional fields

- **WHEN** the AI response omits optional fields (confidence, summary, care_profile)
- **THEN** those fields are `None` in the deserialized `IdentifyResult`

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be deserialized into `IdentifyResponse`
- **THEN** the method returns an error

#### Scenario: IdentifyResult is serializable

- **WHEN** an `IdentifyResult` is serialized to JSON
- **THEN** the output contains `common_name`, `scientific_name`, and any present optional fields

#### Scenario: JSON schema wraps results in suggestions array

- **WHEN** the identify request is built
- **THEN** the `json_schema` response format SHALL define a root object with `suggestions` (array), `rejected` (boolean), and `rejected_reason` (string or null) properties

#### Scenario: Non-plant photo triggers rejection

- **WHEN** `identify` is called with an image that does not contain a plant
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with `rejected: true`, a non-null `rejected_reason`, and an empty `suggestions` array

#### Scenario: Identify prompt includes rejection instruction

- **WHEN** `build_identify_prompt` is called
- **THEN** the prompt text SHALL instruct the model to set `rejected` to `true` when the photo does not show a plant

### Requirement: AI configuration via environment variables

The system SHALL read AI configuration from environment variables: `FLOWL_AI_API_KEY` (required to enable AI, no default), `FLOWL_AI_BASE_URL` (default: `https://api.openai.com/v1`), and `FLOWL_AI_MODEL` (default: `gpt-4.1-mini`).

#### Scenario: All AI env vars set

- **WHEN** `FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, and `FLOWL_AI_MODEL` are set
- **THEN** the AI provider is constructed with the specified values

#### Scenario: Only API key set

- **WHEN** only `FLOWL_AI_API_KEY` is set
- **THEN** the AI provider uses base URL `https://api.openai.com/v1` and model `gpt-4.1-mini`

#### Scenario: No API key set

- **WHEN** `FLOWL_AI_API_KEY` is not set
- **THEN** the AI provider is not constructed and AI features are disabled

### Requirement: Provider lifecycle in AppState

The system SHALL store `Option<Arc<dyn AiProvider>>` in `AppState`. The value MUST be `Some` when `FLOWL_AI_API_KEY` is set and `None` otherwise. The provider instance is created once at startup and shared across all request handlers.

#### Scenario: AI enabled at startup

- **WHEN** the application starts with `FLOWL_AI_API_KEY` set
- **THEN** `AppState` contains `Some(Arc<dyn AiProvider>)`

#### Scenario: AI disabled at startup

- **WHEN** the application starts without `FLOWL_AI_API_KEY`
- **THEN** `AppState` contains `None` for the AI provider field
