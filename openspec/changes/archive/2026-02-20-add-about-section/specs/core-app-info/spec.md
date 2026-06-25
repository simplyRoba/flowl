## ADDED Requirements

### Requirement: App Info Endpoint

The API SHALL expose a `GET /api/info` endpoint that returns application metadata as JSON.

#### Scenario: Info endpoint returns metadata

- **WHEN** a GET request is made to `/api/info`
- **THEN** the server responds with HTTP 200
- **AND** the response body is JSON containing `version`, `repository`, and `license` string fields

#### Scenario: Values match Cargo.toml

- **WHEN** the application is compiled
- **THEN** the `version` field SHALL equal the `version` value from `Cargo.toml`
- **AND** the `repository` field SHALL equal the `repository` value from `Cargo.toml`
- **AND** the `license` field SHALL equal the `license` value from `Cargo.toml`
