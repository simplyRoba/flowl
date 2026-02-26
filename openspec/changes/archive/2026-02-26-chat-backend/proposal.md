## Why

The AI provider trait already supports `identify`, but the `chat` and `summarize` methods are unimplemented stubs. Without the chat backend, the upcoming Chat Drawer UI (Phase 6) has no endpoints to talk to. This phase delivers the streaming chat and summarization API so the full conversational plant-care experience can be built on top.

## What Changes

- Add `chat` method to the OpenAI provider — SSE streaming response, accepts text + optional base64-encoded image, sends plant context as system prompt
- Add `summarize` method to the OpenAI provider — JSON-mode response that condenses a conversation into a 1–3 sentence care journal note
- New `POST /api/ai/chat` endpoint — accepts `{ plant_id, message, image?, history[] }`, streams tokens back via `text/event-stream`
- New `POST /api/ai/summarize` endpoint — accepts `{ plant_id, history[] }`, returns `{ summary }` as JSON
- Plant context builder — loads plant record + recent care events from SQLite and assembles the system prompt sent with every chat request

## Capabilities

### New Capabilities
- `ai/chat`: Streaming chat endpoint and provider method — request/response contract, SSE format, plant context assembly, image support, error handling
- `ai/summarize`: Summarization endpoint and provider method — request/response contract, JSON-mode output, conversation-to-note condensation

### Modified Capabilities
- `ai/provider`: Add `chat` and `summarize` method signatures to the provider trait (currently only defines `identify`)

## Impact

- **Code:** `src/ai/` module — new `chat.rs` and `summarize.rs` handler files, provider trait extension, OpenAI client additions
- **APIs:** Two new HTTP endpoints (`/api/ai/chat`, `/api/ai/summarize`) added to the Axum router
- **Dependencies:** `tokio-stream` (SSE streaming); `reqwest` streaming support (already a dependency)
- **Database:** Read-only — queries `plants` and `care_events` tables to build context; no schema changes
