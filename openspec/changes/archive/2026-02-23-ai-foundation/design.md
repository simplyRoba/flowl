## Context

flowl is a single-crate Rust/Axum service with no AI capabilities. The codebase follows a flat module structure (`src/api/`, `src/config.rs`, `src/state.rs`) with shared `AppState` passed through Axum extractors. Optional subsystems like MQTT use `Option` fields in `AppState` and are gated by environment variables. This change introduces the AI provider foundation following the same pattern.

## Goals / Non-Goals

**Goals:**

- Add AI configuration via environment variables following existing `Config::from_env()` pattern
- Define an `AiProvider` trait that abstracts over OpenAI-compatible APIs (and future providers)
- Wire an optional provider instance into `AppState`
- Expose a status endpoint that mirrors the MQTT status endpoint pattern
- Implement the `identify` method on `OpenAiProvider` (chat and summarize are trait stubs for now)

**Non-Goals:**

- No frontend/UI changes
- No chat or summarize implementation (future phases)
- No `POST /api/ai/identify` endpoint (Phase 3)
- No Ollama or other provider implementations
- No settings UI or database-stored configuration

## Decisions

### 1. New `src/ai/` module, not inside `src/api/`

The AI provider is not an API handler — it's a service layer. Place the trait, types, and OpenAI client in `src/ai/` (`mod.rs`, `provider.rs`, `openai.rs`, `types.rs`). The API handler for `GET /api/ai/status` stays in `src/api/ai.rs` following the existing `src/api/mqtt.rs` pattern.

**Alternative:** Put everything in `src/api/ai.rs`. Rejected because the provider logic is independent of HTTP — it should be testable and usable without Axum.

### 2. `Option<Arc<dyn AiProvider>>` in AppState

Matches the existing `Option<AsyncClient>` pattern used for MQTT. When `FLOWL_AI_API_KEY` is unset, the field is `None` and all AI endpoints can short-circuit with a "disabled" response. `Arc<dyn ...>` because the provider is shared across request handlers and must be `Send + Sync`.

**Alternative:** Always construct a provider and have it fail at call time. Rejected — checking `Option` is cheaper and matches the MQTT pattern.

### 3. `async-trait` crate for the provider trait

Rust stable async trait methods (`async fn` in traits) require `dyn`-compatible workarounds. `async-trait` is the established solution and adds no runtime overhead beyond a heap allocation per call — acceptable for AI API calls that take hundreds of milliseconds.

**Alternative:** Use `-> Pin<Box<dyn Future>>` manually. Rejected — verbose and error-prone for no benefit.

### 4. `reqwest` for HTTP client

`reqwest` is the de facto Rust HTTP client. It supports async, TLS, JSON serialization, and streaming responses. The `rustls-tls` feature avoids linking OpenSSL, keeping the Docker image small.

**Alternative:** `ureq` (sync-only, lighter) or `hyper` (lower-level). Rejected — `ureq` doesn't support async/streaming, `hyper` requires too much boilerplate.

### 5. Single `reqwest::Client` instance in the provider

Reuse one `reqwest::Client` across all AI calls for connection pooling. Constructed once when the provider is created.

### 6. Config fields: three env vars

| Variable | Default | Required |
|----------|---------|----------|
| `FLOWL_AI_API_KEY` | — | Yes (to enable AI) |
| `FLOWL_AI_BASE_URL` | `https://api.openai.com/v1` | No |
| `FLOWL_AI_MODEL` | `gpt-4o-mini` | No |

Use `Option<String>` for the API key (presence = AI enabled). Base URL and model get defaults via `unwrap_or_else`, same as existing config fields.

### 7. Status endpoint mirrors MQTT status

`GET /api/ai/status` returns `{ "enabled": bool, "base_url": "...", "model": "..." }`. When disabled, `base_url` and `model` are omitted (null). Same shape as `MqttStatus` with `status`/`broker`/`topic_prefix`.

## Risks / Trade-offs

- **New dependencies increase binary size** → `reqwest` with `rustls-tls` is the largest addition. Acceptable trade-off for a full-featured HTTP client needed for streaming in later phases. `base64` and `async-trait` are small.
- **`identify` method implemented but no endpoint yet** → The method is testable via unit/integration tests. The endpoint arrives in Phase 3. This avoids a half-working endpoint that could confuse users.
- **Provider trait has `chat` and `summarize` stubs** → They return `unimplemented!()` or a trait-level error. The trait signature is defined now so the interface is stable for Phase 5. Implementations that call these stubs will fail clearly.
