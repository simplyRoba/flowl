## ADDED Requirements

### Requirement: Identify rate limiting

The identify endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Identify request within rate limit

- **WHEN** a valid identify request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Identify request exceeds rate limit

- **WHEN** an identify request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider
