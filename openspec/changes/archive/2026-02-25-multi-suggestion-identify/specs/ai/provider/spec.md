## ADDED Requirements

### Requirement: IdentifyResponse wrapper type

The system SHALL define an `IdentifyResponse` struct containing a `suggestions` field of type `Vec<IdentifyResult>`. The struct SHALL derive `Serialize` and `Deserialize`.

#### Scenario: IdentifyResponse with multiple suggestions

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [{ "common_name": "A", "scientific_name": "B" }, { "common_name": "C", "scientific_name": "D" }] }`
- **THEN** the `suggestions` field SHALL contain 2 `IdentifyResult` entries

#### Scenario: IdentifyResponse serialization round-trip

- **WHEN** an `IdentifyResponse` is serialized to JSON
- **THEN** the output SHALL contain a `suggestions` array with each suggestion's fields

## MODIFIED Requirements

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices) and a `locale` string, encode the images as base64 data URLs, send them to the configured model using structured output (`response_format: { "type": "json_schema" }`) with the `IdentifyResponse` schema, and deserialize the response into an `IdentifyResponse` containing a `suggestions` array of up to 3 `IdentifyResult` entries ranked by confidence. The prompt SHALL instruct the model to provide its top 3 most likely identifications. The prompt SHALL instruct the model to respond in the language matching the given locale for free-text fields (`common_name`, `summary`) while keeping `scientific_name` in Latin. Enum-constrained fields in `care_profile` remain in English by virtue of the JSON schema constraints. The `IdentifyResult` and `CareProfile` types SHALL derive both `Serialize` and `Deserialize` so they can be used as HTTP response bodies.

#### Scenario: Single image identification returns multiple suggestions

- **WHEN** `identify` is called with one image
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with up to 3 suggestions

#### Scenario: Multi-image identification returns multiple suggestions

- **WHEN** `identify` is called with multiple images
- **THEN** all images are included in the same API request as separate image content parts
- **AND** the response SHALL contain up to 3 suggestions

#### Scenario: Suggestions are ranked by confidence

- **WHEN** the AI returns multiple suggestions
- **THEN** the suggestions SHALL be ordered by descending confidence (highest first)

#### Scenario: AI returns fewer than 3 suggestions

- **WHEN** the AI returns only 1 or 2 suggestions
- **THEN** the `IdentifyResponse` SHALL contain only the returned suggestions without error

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be deserialized into `IdentifyResponse`
- **THEN** the method SHALL return an error

#### Scenario: JSON schema wraps results in suggestions array

- **WHEN** the identify request is built
- **THEN** the `json_schema` response format SHALL define a root object with a `suggestions` property of type array, where each item follows the existing `IdentifyResult` schema
