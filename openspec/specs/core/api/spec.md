## Purpose

REST API layer providing JSON request/response handling, error responses, and route mounting under the `/api` prefix.

## Requirements

### Requirement: API Router

The application SHALL mount all REST API routes under the `/api` prefix on the Axum server.

#### Scenario: API route accessible

- **WHEN** a request is made to `/api/plants`
- **THEN** the API router handles the request

#### Scenario: Non-API route falls through

- **WHEN** a request is made to a path not starting with `/api`
- **THEN** the request falls through to the SPA static file handler

### Requirement: JSON Error Responses

The API SHALL return errors as JSON with a consistent structure containing a `message` field and an appropriate HTTP status code.

#### Scenario: Validation error

- **WHEN** a request body is missing required fields
- **THEN** the API responds with HTTP 422 and `{"message": "..."}`

#### Scenario: Not found

- **WHEN** a request references a resource that does not exist
- **THEN** the API responds with HTTP 404 and `{"message": "..."}`

#### Scenario: Invalid JSON body

- **WHEN** a request body contains invalid JSON
- **THEN** the API responds with HTTP 400 and `{"message": "..."}`
