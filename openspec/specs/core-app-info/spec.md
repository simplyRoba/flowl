## Purpose

Application metadata endpoint exposing the running application's version, official source repository, and license information.

## Requirements

### Requirement: App Info Endpoint

The API SHALL expose a `GET /api/info` endpoint that returns application metadata as JSON.

#### Scenario: Info endpoint returns metadata

- **WHEN** a GET request is made to `/api/info`
- **THEN** the server responds with HTTP 200
- **AND** the response body is JSON containing `version`, `repository`, and `license` string fields

#### Scenario: Values identify the application

- **WHEN** a client receives a successful info response
- **THEN** the `version` field is the semantic version of the running application build or release
- **AND** the `repository` field is the URL of the official application source repository
- **AND** the `license` field is the application license identifier
