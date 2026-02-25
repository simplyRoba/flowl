## Purpose

AI provider abstraction layer: trait definition, OpenAI-compatible client implementation, configuration via environment variables, and provider lifecycle in AppState.

## Requirements

### Requirement: AI provider trait

The system SHALL define an `AiProvider` async trait with methods `identify`, `chat`, and `summarize`. The trait MUST be object-safe and `Send + Sync` so it can be shared across async request handlers via `Arc<dyn AiProvider>`.

#### Scenario: Trait is object-safe

- **WHEN** an `AiProvider` implementation is constructed
- **THEN** it can be stored as `Arc<dyn AiProvider>` in shared application state

### Requirement: OpenAI-compatible provider

The system SHALL implement `OpenAiProvider` that communicates with any OpenAI-compatible API endpoint. The provider MUST use a single `reqwest::Client` instance for connection pooling and MUST send requests to `{base_url}/chat/completions`.

#### Scenario: Provider targets configured base URL

- **WHEN** `OpenAiProvider` is constructed with base URL `https://api.openai.com/v1`
- **THEN** API requests are sent to `https://api.openai.com/v1/chat/completions`

#### Scenario: Provider targets custom base URL

- **WHEN** `OpenAiProvider` is constructed with base URL `http://localhost:11434/v1`
- **THEN** API requests are sent to `http://localhost:11434/v1/chat/completions`

### Requirement: IdentifyResponse wrapper type

The system SHALL define an `IdentifyResponse` struct containing a `suggestions` field of type `Vec<IdentifyResult>`. The struct SHALL derive `Serialize` and `Deserialize`.

#### Scenario: IdentifyResponse with multiple suggestions

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [{ "common_name": "A", "scientific_name": "B" }, { "common_name": "C", "scientific_name": "D" }] }`
- **THEN** the `suggestions` field SHALL contain 2 `IdentifyResult` entries

#### Scenario: IdentifyResponse serialization round-trip

- **WHEN** an `IdentifyResponse` is serialized to JSON
- **THEN** the output SHALL contain a `suggestions` array with each suggestion's fields

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices) and a `locale` string, encode the images as base64 data URLs, send them to the configured model using structured output (`response_format: { "type": "json_schema" }`) with the `IdentifyResponse` schema, and deserialize the response into an `IdentifyResponse` containing a `suggestions` array of up to 3 `IdentifyResult` entries ranked by confidence. The prompt SHALL instruct the model to provide its top 3 most likely identifications. The prompt SHALL instruct the model to respond in the language matching the given locale for free-text fields (`common_name`, `summary`) while keeping `scientific_name` in Latin. Enum-constrained fields in `care_profile` remain in English by virtue of the JSON schema constraints. The `IdentifyResult` and `CareProfile` types SHALL derive both `Serialize` and `Deserialize` so they can be used as HTTP response bodies.

#### Scenario: Single image identification returns multiple suggestions

- **WHEN** `identify` is called with one image
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with up to 3 suggestions

#### Scenario: Multi-image identification returns multiple suggestions

- **WHEN** `identify` is called with multiple images
- **THEN** all images are included in the same API request as separate image content parts
- **AND** the response SHALL contain up to 3 suggestions

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
- **THEN** the `json_schema` response format SHALL define a root object with a `suggestions` property of type array, where each item follows the existing `IdentifyResult` schema

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
