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

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices), encode them as base64 data URLs, send them to the configured model using JSON mode (`response_format: { "type": "json_object" }`), and deserialize the response into an `IdentifyResult` containing `common_name`, `scientific_name`, optional `confidence`, optional `summary`, and optional `care_profile`.

#### Scenario: Single image identification

- **WHEN** `identify` is called with one image
- **THEN** the image is sent as a base64-encoded data URL in the API request
- **AND** the response is deserialized into an `IdentifyResult`

#### Scenario: Multi-image identification

- **WHEN** `identify` is called with multiple images
- **THEN** all images are included in the same API request as separate image content parts

#### Scenario: AI returns incomplete optional fields

- **WHEN** the AI response omits optional fields (confidence, summary, care_profile)
- **THEN** those fields are `None` in the deserialized `IdentifyResult`

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be deserialized into `IdentifyResult`
- **THEN** the method returns an error

### Requirement: AI configuration via environment variables

The system SHALL read AI configuration from environment variables: `FLOWL_AI_API_KEY` (required to enable AI, no default), `FLOWL_AI_BASE_URL` (default: `https://api.openai.com/v1`), and `FLOWL_AI_MODEL` (default: `gpt-4o-mini`).

#### Scenario: All AI env vars set

- **WHEN** `FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, and `FLOWL_AI_MODEL` are set
- **THEN** the AI provider is constructed with the specified values

#### Scenario: Only API key set

- **WHEN** only `FLOWL_AI_API_KEY` is set
- **THEN** the AI provider uses base URL `https://api.openai.com/v1` and model `gpt-4o-mini`

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
