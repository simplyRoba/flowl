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

### Requirement: Identify rate limiting

The identify endpoint SHALL check the global AI rate limiter before processing the request. If the limit is exceeded, the endpoint SHALL return HTTP 429 with error code `AI_RATE_LIMITED` without forwarding anything to the AI provider.

#### Scenario: Identify request within rate limit

- **WHEN** a valid identify request is sent and the rate limit has not been exceeded
- **THEN** the request SHALL be processed normally

#### Scenario: Identify request exceeds rate limit

- **WHEN** an identify request is sent and the rate limit has been exceeded
- **THEN** the endpoint SHALL return HTTP 429 with `{"code": "AI_RATE_LIMITED", "message": "..."}`
- **AND** no request SHALL be sent to the AI provider

### Requirement: Identify endpoint returns 422 when AI rejects non-plant photo

The endpoint SHALL check the `IdentifyResponse` for `rejected == true` after a successful AI provider call. When the AI rejects the photo, the endpoint SHALL log the `rejected_reason` at `warn` level and return HTTP 422 with error code `AI_IDENTIFY_NOT_A_PLANT` using the established `ApiError` pattern. The `default_message` for this code SHALL be `"The photo does not appear to contain a plant"`.

#### Scenario: AI rejects a non-plant photo

- **WHEN** a POST request is made to `/api/ai/identify` with a valid photo
- **AND** the AI provider returns an `IdentifyResponse` with `rejected: true` and `rejected_reason: "This is a coffee mug"`
- **THEN** the response status is 422
- **AND** the body contains `{"code": "AI_IDENTIFY_NOT_A_PLANT", "message": "The photo does not appear to contain a plant"}`

#### Scenario: AI rejects but reason is logged

- **WHEN** the AI provider returns `rejected: true` with a `rejected_reason`
- **THEN** the `rejected_reason` SHALL be logged at `warn` level

#### Scenario: AI accepts a plant photo

- **WHEN** a POST request is made to `/api/ai/identify` with a valid photo
- **AND** the AI provider returns an `IdentifyResponse` with `rejected: false`
- **THEN** the response status is 200
- **AND** the body contains `{ "suggestions": [ ... ] }` as before

### Requirement: Internal error API response

The API error type SHALL include an `InternalError` variant that maps to HTTP 500 with a JSON `{"message": "..."}` body, used for unexpected server-side failures.

#### Scenario: Internal error response format

- **WHEN** an internal error occurs during request processing
- **THEN** the response status is 500
- **AND** the body is `{"message": "<error description>"}`
