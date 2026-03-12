## 1. Configuration

- [x] 1.1 Add `ai_rate_limit: u32` field to `Config` in `src/config.rs`, parsed from `FLOWL_AI_RATE_LIMIT` env var with default 10. Value 0 disables rate limiting.
- [x] 1.2 Add config tests for default value, custom value, and 0 (disabled)

## 2. Rate Limiter State

- [x] 2.1 Add `AiRateLimiter` struct in `src/state.rs` with `AtomicU32` counter and `AtomicU64` window start timestamp. Implement `check()` method that returns `true` if request is allowed, `false` if rate limited. Use `compare_exchange` for safe window reset.
- [x] 2.2 Add `ai_rate_limiter: Option<AiRateLimiter>` to `AppState` (None when limit is 0 / disabled)
- [x] 2.3 Add unit tests for `AiRateLimiter`: within limit, at limit, over limit, window reset, disabled (None)

## 3. Error Handling

- [x] 3.1 Add `TooManyRequests(&'static str)` variant to `ApiError` in `src/api/error.rs` mapping to HTTP 429
- [x] 3.2 Add `AI_RATE_LIMITED` to `default_message` with message "Too many AI requests, please wait"
- [x] 3.3 Add error variant test for `TooManyRequests`

## 4. AI Endpoint Integration

- [x] 4.1 Add `check_ai_rate_limit` helper function in `src/api/ai.rs` that takes `&AppState` and returns `Result<(), ApiError>`
- [x] 4.2 Call `check_ai_rate_limit` at the top of `chat`, `identify_plant`, and `summarize` handlers
- [x] 4.3 Add integration tests: request succeeds within limit, returns 429 with `AI_RATE_LIMITED` code when exceeded

## 5. UI Error Code

- [x] 5.1 Add `AI_RATE_LIMITED` translation to `en.ts`, `de.ts`, `es.ts` errorCode dictionaries

## 6. Documentation & Verification

- [x] 6.1 Add `FLOWL_AI_RATE_LIMIT` to the config table in `README.md`
- [x] 6.2 Run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `npm run check --prefix ui`, `npm run lint --prefix ui`, `npm run format:check --prefix ui`
