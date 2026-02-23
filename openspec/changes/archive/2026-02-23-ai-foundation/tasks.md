## 1. Dependencies and Configuration

- [x] 1.1 Add `reqwest` (with `rustls-tls` and `json` features), `async-trait`, `base64`, and `tokio-stream` to `Cargo.toml`
- [x] 1.2 Add `ai_api_key: Option<String>`, `ai_base_url: String`, and `ai_model: String` fields to `Config` struct, reading from `FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL` (default `https://api.openai.com/v1`), `FLOWL_AI_MODEL` (default `gpt-4o-mini`)
- [x] 1.3 Add config tests: defaults when no AI env vars set, custom values when all set, API key absent results in `None`

## 2. AI Provider Trait and Types

- [x] 2.1 Create `src/ai/` module with `mod.rs`, `provider.rs`, `types.rs`; register `pub mod ai` in `src/lib.rs`
- [x] 2.2 Define `AiProvider` async trait in `provider.rs` with methods `identify`, `chat`, `summarize`
- [x] 2.3 Define types in `types.rs`: `IdentifyResult` (common_name, scientific_name, confidence, summary, care_profile), `CareProfile` (watering_interval_days, light_needs, difficulty, pet_safety, growth_speed, soil_type, soil_moisture), `ChatMessage`, `ChatResponseStream`
- [x] 2.4 Add unit tests for `IdentifyResult` deserialization: complete response, missing optional fields, unparseable response

## 3. OpenAI Provider Implementation

- [x] 3.1 Create `src/ai/openai.rs` with `OpenAiProvider` struct holding `reqwest::Client`, `api_key`, `base_url`, `model`
- [x] 3.2 Implement `identify` method: encode images as base64 data URLs, build chat completions request with JSON mode, deserialize response into `IdentifyResult`
- [x] 3.3 Add `chat` and `summarize` as stubs returning `unimplemented!()` error
- [x] 3.4 Add unit tests for base URL construction (`{base_url}/chat/completions`), request payload structure (JSON mode, image content parts)

## 4. AppState Integration

- [x] 4.1 Add `ai_provider: Option<Arc<dyn AiProvider>>` field to `AppState`
- [x] 4.2 Wire up provider construction in `main.rs`: if `config.ai_api_key` is `Some`, create `OpenAiProvider` and wrap in `Arc`; otherwise `None`
- [x] 4.3 Store `ai_base_url` and `ai_model` on `AppState` (needed by the status endpoint when provider is `None`)

## 5. Status Endpoint

- [x] 5.1 Create `src/api/ai.rs` with `get_ai_status` handler returning `AiStatus { enabled, base_url, model }`; register in `src/api/mod.rs`
- [x] 5.2 Add route `/ai/status` to the API router in `src/api/mod.rs`
- [x] 5.3 Add tests for status endpoint: returns `enabled: true` with base_url and model when AI configured, returns `enabled: false` with nulls when AI not configured

## 6. Verification

- [x] 6.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` — fix any issues
- [x] 6.2 Update README.md with AI configuration env vars (`FLOWL_AI_API_KEY`, `FLOWL_AI_BASE_URL`, `FLOWL_AI_MODEL`) if a configuration section exists
