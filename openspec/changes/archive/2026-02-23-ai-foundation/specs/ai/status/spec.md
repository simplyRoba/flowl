## ADDED Requirements

### Requirement: AI status endpoint

The system SHALL expose `GET /api/ai/status` returning a JSON object with fields `enabled` (boolean), `base_url` (string or null), and `model` (string or null).

#### Scenario: AI is enabled

- **WHEN** a GET request is made to `/api/ai/status` and AI is configured
- **THEN** the response status is 200
- **AND** the body is `{ "enabled": true, "base_url": "<configured URL>", "model": "<configured model>" }`

#### Scenario: AI is disabled

- **WHEN** a GET request is made to `/api/ai/status` and AI is not configured
- **THEN** the response status is 200
- **AND** the body is `{ "enabled": false, "base_url": null, "model": null }`
