## Why

AI endpoints (`/api/ai/chat`, `/api/ai/identify`, `/api/ai/summarize`) forward requests to external AI providers without any rate limiting. Since the service has no authentication, any client on the network can burn through API credits with rapid requests. This is review item B9.

## What Changes

- Add a global in-memory rate limiter scoped to AI endpoints only
- Configurable via `FLOWL_AI_RATE_LIMIT` env var (requests per minute, default 10)
- Return HTTP 429 with structured error code `AI_RATE_LIMITED` when exceeded
- Rate limit check runs before the AI provider call, so no credits are spent on rejected requests
- `/api/ai/status` is excluded (read-only, no cost)

## Capabilities

### New Capabilities

_None — this extends existing capabilities._

### Modified Capabilities

- `core/api`: Add `TooManyRequests` (429) error variant and `AI_RATE_LIMITED` error code to the catalog
- `ai/chat`: Add rate limit check before processing
- `ai/identify`: Add rate limit check before processing
- `ai/summarize`: Add rate limit check before processing

## Impact

- `src/config.rs`: New `ai_rate_limit` field
- `src/state.rs`: Rate limiter state (atomics) in `AppState`
- `src/api/error.rs`: New `TooManyRequests` variant, `AI_RATE_LIMITED` code
- `src/api/ai.rs`: Rate limit check at top of chat/identify/summarize handlers
- UI i18n: `AI_RATE_LIMITED` translation in en/de/es
- No new dependencies — uses `std::sync::atomic` only
