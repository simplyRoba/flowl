## ADDED Requirements

### Requirement: Chat rate limiting

The chat endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Chat request within rate limit

- **WHEN** a valid chat request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Chat request exceeds rate limit

- **WHEN** a chat request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider
