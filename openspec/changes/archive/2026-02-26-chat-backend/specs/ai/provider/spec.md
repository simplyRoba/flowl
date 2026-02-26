## MODIFIED Requirements

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

The `summarize` implementation SHALL send a non-streaming request with `response_format: { "type": "json_object" }`, deserialize the response, and extract the `summary` field.

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
