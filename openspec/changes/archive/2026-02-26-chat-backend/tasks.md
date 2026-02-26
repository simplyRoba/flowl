## 1. Dependencies & Types

- [x] 1.1 Add `stream` feature to `reqwest` in `Cargo.toml`
- [x] 1.2 Update `ChatMessage` struct: add `Serialize`/`Deserialize` derives and optional `image: Option<String>` field
- [x] 1.3 Update `AiProvider` trait: change `chat` signature to accept `system_prompt`, `messages`, `image`, `locale`; change `summarize` signature to accept `system_prompt`, `messages`, `locale`
- [x] 1.4 Update `OpenAiProvider` stub signatures to match the new trait (keep `unimplemented!()` for now so it compiles)

## 2. Plant Context Builder

- [x] 2.1 Implement `build_plant_context` function in `src/api/ai.rs` — query plant record + last 20 care events, serialize to JSON string
- [x] 2.2 Write unit tests for context builder: plant with events, plant with no events, plant with NULL optional fields

## 3. Chat System Prompt

- [x] 3.1 Implement `build_chat_system_prompt` function — compose the system prompt with identity, instructions, plant context JSON, and locale directive
- [x] 3.2 Implement `build_summarize_system_prompt` function — compose the summarize prompt with plant name/species, condensation instructions, and locale directive
- [x] 3.3 Write unit tests for both prompt builders: verify plant context is embedded, locale instructions are set correctly

## 4. OpenAI Provider — Chat Method

- [x] 4.1 Implement `chat` on `OpenAiProvider` — build the messages array with system prompt, history (including historical images), and current message with optional image; send with `stream: true`
- [x] 4.2 Implement SSE stream parsing — spawn background task that reads `reqwest` byte stream, splits on `data:` lines, extracts `choices[0].delta.content`, skips `[DONE]` and empty lines, sends tokens through `mpsc` channel
- [x] 4.3 Write unit tests for SSE line parsing logic (delta extraction, [DONE] handling, empty lines, malformed chunks)

## 5. OpenAI Provider — Summarize Method

- [x] 5.1 Implement `summarize` on `OpenAiProvider` — build messages with system prompt + history, send with `response_format: { "type": "json_object" }`, extract `summary` field from response
- [x] 5.2 Write unit tests for summarize response parsing: valid `{"summary":"..."}`, missing field, invalid JSON

## 6. HTTP Endpoints

- [x] 6.1 Define `ChatRequest` and `SummarizeRequest` deserialization structs in `src/api/ai.rs`
- [x] 6.2 Implement `POST /api/ai/chat` handler — validate provider exists, validate plant exists, decode optional base64 image, build context + system prompt, call provider, wrap `ChatResponseStream` in Axum `Sse` response with JSON event formatting (`delta`/`done`/`error`)
- [x] 6.3 Implement `POST /api/ai/summarize` handler — validate provider exists, validate plant exists, validate history not empty, build context + system prompt, call provider, return `{"summary":"..."}`
- [x] 6.4 Register both routes in `src/api/mod.rs` with `DefaultBodyLimit::max(30 * 1024 * 1024)` on the chat route

## 7. Quality Checks

- [x] 7.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` — fix any issues
- [x] 7.2 Run `cd ui && npm run check` — fix any issues
- [x] 7.3 Manual smoke test via curl: `POST /api/ai/chat` with a real plant_id and verify SSE stream; `POST /api/ai/summarize` and verify JSON response
