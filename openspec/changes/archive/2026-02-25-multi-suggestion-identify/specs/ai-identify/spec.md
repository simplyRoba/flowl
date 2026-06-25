## MODIFIED Requirements

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
