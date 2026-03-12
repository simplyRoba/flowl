## Context

AI endpoints forward requests to external providers (OpenAI-compatible APIs) without any rate limiting. The service has no authentication, so any client on the network can make unlimited AI calls, burning through API credits. This is a self-hosted single-binary service typically serving one user, but protection against runaway clients or bugs is needed.

## Goals / Non-Goals

**Goals:**
- Protect AI API credits from excessive usage
- Return proper error codes when rate limited (consistent with existing error pattern)
- Make the limit configurable via environment variable
- Zero external dependencies — use only `std::sync::atomic`

**Non-Goals:**
- Per-user or per-IP rate limiting (no auth system exists)
- Rate limiting non-AI endpoints (they're local/cheap)
- Persistent rate limit state across restarts
- Distributed rate limiting

## Decisions

### D1: Global atomic counter with minute window

**Choice**: Single `AtomicU32` counter + `AtomicU64` window timestamp in `AppState`.

**Why**: The service is single-process, single-user. A global counter is the simplest thing that works. No mutex needed — atomic compare-and-swap handles concurrent requests safely.

**Alternatives considered**:
- **tower-governor / tower rate limit middleware**: Adds a dependency, more complex than needed for 3 endpoints. The project prefers minimal dependencies.
- **Token bucket**: More sophisticated but unnecessary for a single-user app with minute-granularity limits.
- **Per-IP tracking**: No auth exists, and in typical deployment (behind reverse proxy), all requests come from the same IP anyway.

**Mechanics**: On each AI request, check if the current minute matches the stored window. If yes, increment counter and reject if over limit. If new minute, reset counter to 1.

### D2: Default limit of 10 requests per minute

**Choice**: Default to 10 req/min, configurable via `FLOWL_AI_RATE_LIMIT`.

**Why**: Realistic user flows — chat takes seconds per exchange (3-5 messages/min max), identify is ~1/min, summarize is ~1/conversation. 10/min is generous for normal use but catches runaway loops. Setting to 0 disables the limiter.

### D3: Check before provider call

**Choice**: Rate limit check runs as the first thing in each AI handler, before any AI provider interaction.

**Why**: Rejected requests cost nothing — no API credits spent, no provider latency. The check is a few atomic operations (~nanoseconds).

### D4: New `TooManyRequests` error variant

**Choice**: Add `TooManyRequests(&'static str)` to `ApiError` enum mapping to HTTP 429.

**Why**: Follows the existing pattern exactly. Every HTTP status code used by the API has its own `ApiError` variant. 429 is the standard status for rate limiting.

## Risks / Trade-offs

- **[No per-user fairness]** → Acceptable: single-user service. If multi-user support is added later, rate limiting would need to be revisited alongside authentication.
- **[Counter resets on restart]** → Acceptable: restarts are infrequent and the window is only 1 minute. No need for persistence.
- **[Atomic race on window reset]** → Mitigated: use `compare_exchange` for the window timestamp so only one thread resets the counter. Other threads retry and see the new window.
