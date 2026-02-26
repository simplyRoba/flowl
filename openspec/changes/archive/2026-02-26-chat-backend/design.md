## Context

flowl already has an AI provider trait (`AiProvider`) with a working `identify` method behind an OpenAI-compatible client. The `chat` and `summarize` methods exist as `unimplemented!()` stubs. The provider is stored as `Option<Arc<dyn AiProvider>>` in `AppState`, which means all AI endpoints can gate on its presence. Two new HTTP endpoints are needed so the upcoming Chat Drawer UI (Phase 6) has backend support for conversational plant care and journal-note generation.

Key constraints:
- Single crate, single Docker image — keep dependencies minimal
- `reqwest` is already present (used by `identify`) but only has `rustls-tls` and `json` features — streaming needs the `stream` feature added
- Axum's built-in `axum::response::sse` module provides SSE support without additional crates
- `tokio-stream` is already in `Cargo.toml`

## Goals / Non-Goals

**Goals:**
- Implement `chat` and `summarize` on `OpenAiProvider` using the same model/API key as `identify`
- Expose `POST /api/ai/chat` (SSE streaming) and `POST /api/ai/summarize` (JSON) endpoints
- Build plant context (plant record + last 20 care events) for the system prompt
- Support an optional base64-encoded image in chat messages (vision model)
- Locale-aware responses (reuse the existing locale lookup from `user_settings`)

**Non-Goals:**
- Chat history persistence (session-only, managed by the frontend)
- Rate limiting or token budget tracking
- Frontend/UI work (Phase 6)
- New care event type `ai-consultation` (Phase 7)
- Multi-provider support (future)

## Decisions

### 1. SSE via Axum's built-in `axum::response::sse`

Use Axum's `Sse` response type wrapping a `tokio_stream::ReceiverStream`. The OpenAI provider sends `stream: true` in the request and reads chunked response lines (`data: {...}` SSE format from the OpenAI API), extracts delta tokens, and forwards them through a `tokio::sync::mpsc` channel.

**Why not raw `text/event-stream` with manual formatting?** Axum's `Sse` handles encoding, keep-alive, and correct headers. It's already part of axum (no extra feature flag needed).

**Why `mpsc` channel → `ReceiverStream`?** This is the same pattern already defined in `ChatResponseStream` (`tokio_stream::wrappers::ReceiverStream<Result<String, String>>`). The provider spawns a background task that reads from `reqwest`'s streaming response and pushes deltas into the sender half. The HTTP handler wraps the receiver in `Sse`.

### 2. Provider trait signature changes

Current stubs:
```rust
async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponseStream, ...>;
async fn summarize(&self, text: &str) -> Result<String, ...>;
```

Updated signatures:
```rust
async fn chat(
    &self,
    system_prompt: &str,
    messages: &[ChatMessage],
    image: Option<&[u8]>,
    locale: &str,
) -> Result<ChatResponseStream, ...>;

async fn summarize(
    &self,
    system_prompt: &str,
    messages: &[ChatMessage],
    locale: &str,
) -> Result<String, ...>;
```

**Rationale:** The handler builds the system prompt from plant context and passes it in — the provider doesn't need to know about plants. `ChatMessage` already has `role` and `content`. The optional `image` is only for the latest user message. Locale drives response language (same pattern as `identify`).

### 3. ChatMessage gets optional image field

Extend `ChatMessage` to carry an optional image:
```rust
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub image: Option<String>,  // base64-encoded, only for user messages
}
```

The `image` field on `ChatMessage` stores historical images from prior turns. The separate `image: Option<&[u8]>` parameter on the trait method carries the *current* message's image as raw bytes (decoded from base64 by the handler). The provider encodes the current image to base64 for the API call. For history messages with images, the provider uses them as-is (already base64).

### 4. Plant context builder as a standalone function

A free function in `src/api/ai.rs` (or a small helper module) that takes `&SqlitePool` and `plant_id` and returns a serialized JSON string for the system prompt. Queries:
- `plants` table: name, species, location, light_needs, watering_interval_days, watering status fields, soil/difficulty/pet_safety, notes
- `care_events` table: last 20 events for the plant, ordered by `occurred_at DESC`

**Why not a method on `AppState`?** Keeps `AppState` as a plain data struct. The context builder is specific to AI chat and doesn't belong on the shared state.

**System prompt structure:**
```
You are flowl, a plant care assistant. You help users with plant health, watering,
and general care questions. Be concise and practical.

Plant context:
{json with plant fields + recent care events}

Respond in {language based on locale}.
```

### 5. Summarize uses the same chat completions endpoint with JSON mode

The `summarize` method sends the full conversation history with a system prompt asking for a 1–3 sentence summary. Uses `response_format: { "type": "json_object" }` (simpler than `json_schema` — the output is just `{ "summary": "..." }`). This avoids defining a strict schema for a trivial shape.

**Why `json_object` not `json_schema`?** The summarize response is a single string field. `json_object` is simpler and more widely supported across OpenAI-compatible providers.

### 6. Request body format for `/api/ai/chat`

```json
{
  "plant_id": 42,
  "message": "The lower leaves are turning yellow",
  "image": "<base64 or null>",
  "history": [
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." }
  ]
}
```

JSON body (not multipart) — the frontend sends images as base64. This keeps the API simple since chat images are ephemeral and typically phone-camera shots (a few MB at most). The existing `DefaultBodyLimit` of 30MB (used by identify) is reasonable here too.

### 7. SSE event format

```
data: {"delta":"Based on "}
data: {"delta":"your photo..."}
data: {"done":true}
```

Each SSE event is a JSON object with either a `delta` (text chunk) or `done: true` (stream end). On error mid-stream, send `data: {"error":"..."}` and close. This matches the format specified in PLAN.md.

### 8. reqwest streaming feature

Add `"stream"` to reqwest's features in `Cargo.toml`:
```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
```

This enables `response.bytes_stream()` for reading the OpenAI SSE response as an async byte stream.

## Risks / Trade-offs

**[No token/cost limits]** → Users can run up API costs with long conversations. Mitigation: out of scope for this phase; the frontend controls history length. A future phase could add token budgets.

**[OpenAI API streaming format differences across providers]** → Some OpenAI-compatible providers (e.g., older Ollama versions) may format SSE chunks slightly differently. Mitigation: parse the `data:` lines defensively, skip empty lines and `[DONE]` markers, log unparseable chunks rather than erroring.

**[Large base64 images in JSON body]** → A 5MB photo becomes ~6.7MB base64. With history containing multiple images, request bodies can grow large. Mitigation: the `DefaultBodyLimit` of 30MB is sufficient; the frontend should strip images from history entries older than the most recent turn.

**[System prompt context window usage]** → The plant context + 20 care events consume tokens. Mitigation: keep the context JSON minimal (no full timestamps, just dates; no redundant fields). 20 events is a reasonable cap.
