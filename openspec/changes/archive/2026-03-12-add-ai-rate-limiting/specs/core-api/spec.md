## ADDED Requirements

### Requirement: Rate limit error response

The API SHALL support a `TooManyRequests` error variant that returns HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}` when the AI rate limit is exceeded.

#### Scenario: Rate limit exceeded

- **WHEN** an AI endpoint receives a request that exceeds the configured rate limit
- **THEN** the API responds with HTTP 429 and `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be forwarded to the AI provider
