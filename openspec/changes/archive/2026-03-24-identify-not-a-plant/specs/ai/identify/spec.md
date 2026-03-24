## ADDED Requirements

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
