## Why

flowl has no AI capabilities. Phase 1 of the AI integration plan adds the backend foundation: configuration, an abstracted provider interface, and a status endpoint. This enables all subsequent AI features (identification, chat, summarize) to build on a stable, tested base without touching the frontend yet.

## What Changes

- Add AI-related environment variables to `Config` (`FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, `FLOWL_AI_MODEL`) — single model for all AI tasks (vision + text)
- Define an `AiProvider` async trait with methods: `identify`, `chat`, `summarize`
- Implement `OpenAiProvider` with the `identify` method (vision model, JSON mode, multi-image support)
- Store `Option<Arc<dyn AiProvider>>` in `AppState` — `None` when no API key is set
- Add `GET /api/ai/status` endpoint returning enabled/disabled state, base URL, and model name
- Add new dependencies: `reqwest` (HTTP client), `async-trait`, `base64` (image encoding), `tokio-stream` (SSE streaming for later phases)

## Capabilities

### New Capabilities

- `ai/provider`: AI provider trait, OpenAI-compatible client implementation, configuration via env vars, and provider lifecycle in AppState
- `ai/status`: `GET /api/ai/status` endpoint exposing whether AI is enabled and which model is configured

### Modified Capabilities

- `core/foundation`: AppState gains an optional AI provider field; Config gains AI env vars (`FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, `FLOWL_AI_MODEL`)

## Impact

- **Code**: New `src/ai/` module (provider trait, OpenAI client, config fields). `AppState` and `Config` structs modified.
- **API**: One new endpoint (`GET /api/ai/status`). No changes to existing endpoints.
- **Dependencies**: `reqwest`, `async-trait`, `base64`, `tokio-stream` added to `Cargo.toml`.
- **Deployment**: No behavior change unless `FLOWL_AI_API_KEY` is set. Fully backwards-compatible — existing deployments unaffected.
