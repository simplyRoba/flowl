## Purpose

HTTP endpoint for AI-powered plant identification: accepts multipart photo uploads, validates input, forwards images to the AI provider, and returns identification results as JSON.

## Requirements

### Requirement: Identify endpoint accepts multipart photo uploads

The system SHALL expose `POST /api/ai/identify` accepting `multipart/form-data` with 1–3 image files in fields named `photos`. Accepted content types are `image/jpeg`, `image/png`, and `image/webp`. The endpoint SHALL read the user's locale from `user_settings` and forward it along with the images to the configured AI provider's `identify` method. If the locale cannot be read, it SHALL default to `en`. The result is returned as JSON containing a `suggestions` array of up to 3 `IdentifyResult` objects.

#### Scenario: Single photo identification

- **WHEN** a POST request is made to `/api/ai/identify` with one JPEG image in the `photos` field
- **THEN** the response status is 200
- **AND** the body contains `{ "suggestions": [ ... ] }` with each entry containing `common_name`, `scientific_name`, and optional `confidence`, `summary`, and `care_profile` fields

#### Scenario: Multiple photo identification

- **WHEN** a POST request is made to `/api/ai/identify` with 2–3 images in `photos` fields
- **THEN** all images are forwarded to the AI provider in a single identify call
- **AND** the response status is 200
- **AND** the body contains `{ "suggestions": [ ... ] }`

#### Scenario: Response contains multiple suggestions

- **WHEN** a POST request is made to `/api/ai/identify` with valid photos
- **THEN** the response body SHALL contain a `suggestions` array with 1–3 identification results ranked by confidence

### Requirement: Identify endpoint validates input

The endpoint SHALL reject requests that do not contain valid photo uploads.

#### Scenario: No photos provided

- **WHEN** a POST request is made to `/api/ai/identify` with no file fields
- **THEN** the response status is 422
- **AND** the body contains `{"message": "..."}`

#### Scenario: Invalid content type

- **WHEN** a POST request is made to `/api/ai/identify` with a file that is not JPEG, PNG, or WebP
- **THEN** the response status is 422
- **AND** the body contains `{"message": "..."}`

### Requirement: Identify endpoint returns 503 when AI is disabled

The endpoint SHALL return HTTP 503 when the AI provider is not configured.

#### Scenario: AI not configured

- **WHEN** a POST request is made to `/api/ai/identify` and `FLOWL_AI_API_KEY` is not set
- **THEN** the response status is 503
- **AND** the body contains `{"message": "..."}`

### Requirement: Identify endpoint handles AI provider errors

The endpoint SHALL return HTTP 500 when the AI provider fails (network error, non-200 upstream response, or unparseable AI output).

#### Scenario: AI API returns an error

- **WHEN** a POST request is made to `/api/ai/identify` with valid photos
- **AND** the AI provider returns an error
- **THEN** the response status is 500
- **AND** the body contains `{"message": "..."}`

### Requirement: Identify endpoint body size limit

The endpoint SHALL enforce a 30 MB request body size limit to accommodate up to 3 photos with multipart overhead.

#### Scenario: Request exceeds body size limit

- **WHEN** a POST request is made to `/api/ai/identify` with a body exceeding 30 MB
- **THEN** the request is rejected before handler execution

### Requirement: Internal error API response

The API error type SHALL include an `InternalError` variant that maps to HTTP 500 with a JSON `{"message": "..."}` body, used for unexpected server-side failures.

#### Scenario: Internal error response format

- **WHEN** an internal error occurs during request processing
- **THEN** the response status is 500
- **AND** the body is `{"message": "<error description>"}`
