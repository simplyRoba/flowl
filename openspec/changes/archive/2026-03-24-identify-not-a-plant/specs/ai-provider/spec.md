## MODIFIED Requirements

### Requirement: IdentifyResponse wrapper type

The system SHALL define an `IdentifyResponse` struct containing a `suggestions` field of type `Vec<IdentifyResult>`, a `rejected` field of type `Option<bool>`, and a `rejected_reason` field of type `Option<String>`. The struct SHALL derive `Serialize` and `Deserialize`. When `rejected` is `true`, `suggestions` SHALL be empty and `rejected_reason` SHALL contain the AI's explanation. When `rejected` is `false` or `None`, `suggestions` SHALL contain 1–3 results and `rejected_reason` SHALL be `None`.

#### Scenario: IdentifyResponse with multiple suggestions

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [{ "common_name": "A", "scientific_name": "B" }, { "common_name": "C", "scientific_name": "D" }], "rejected": false, "rejected_reason": null }`
- **THEN** the `suggestions` field SHALL contain 2 `IdentifyResult` entries
- **AND** `rejected` SHALL be `Some(false)`

#### Scenario: IdentifyResponse serialization round-trip

- **WHEN** an `IdentifyResponse` is serialized to JSON
- **THEN** the output SHALL contain a `suggestions` array with each suggestion's fields

#### Scenario: IdentifyResponse with rejection

- **WHEN** an `IdentifyResponse` is deserialized from JSON `{ "suggestions": [], "rejected": true, "rejected_reason": "This is a coffee mug" }`
- **THEN** `rejected` SHALL be `Some(true)`
- **AND** `rejected_reason` SHALL be `Some("This is a coffee mug")`
- **AND** `suggestions` SHALL be empty

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices) and a `locale` string, encode the images as base64 data URLs, send them to the configured model using structured output (`response_format: { "type": "json_schema" }`) with the `IdentifyResponse` schema, and deserialize the response into an `IdentifyResponse` containing a `suggestions` array of up to 3 `IdentifyResult` entries ranked by confidence. The prompt SHALL instruct the model to provide its top 3 most likely identifications. The prompt SHALL instruct the model to respond in the language matching the given locale for free-text fields (`common_name`, `summary`) while keeping `scientific_name` in Latin. Enum-constrained fields in `care_profile` remain in English by virtue of the JSON schema constraints. The `IdentifyResult` and `CareProfile` types SHALL derive both `Serialize` and `Deserialize` so they can be used as HTTP response bodies.

The JSON schema sent to the AI SHALL include top-level `rejected` (boolean, required) and `rejected_reason` (string or null, required) fields alongside the existing `suggestions` array. The prompt SHALL instruct the model to set `rejected` to `true` with a brief `rejected_reason` and an empty `suggestions` array when the photo does not contain a plant. When the photo does contain a plant, the model SHALL set `rejected` to `false`, `rejected_reason` to `null`, and populate `suggestions` as before.

#### Scenario: Single image identification returns multiple suggestions

- **WHEN** `identify` is called with one image of a plant
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with `rejected: false` and up to 3 suggestions

#### Scenario: Multi-image identification returns multiple suggestions

- **WHEN** `identify` is called with multiple images of a plant
- **THEN** all images are included in the same API request as separate image content parts
- **AND** the response SHALL contain `rejected: false` and up to 3 suggestions

#### Scenario: Suggestions are ranked by confidence

- **WHEN** the AI returns multiple suggestions
- **THEN** the suggestions SHALL be ordered by descending confidence (highest first)

#### Scenario: AI returns fewer than 3 suggestions

- **WHEN** the AI returns only 1 or 2 suggestions
- **THEN** the `IdentifyResponse` SHALL contain only the returned suggestions without error

#### Scenario: AI returns incomplete optional fields

- **WHEN** the AI response omits optional fields (confidence, summary, care_profile)
- **THEN** those fields are `None` in the deserialized `IdentifyResult`

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be deserialized into `IdentifyResponse`
- **THEN** the method returns an error

#### Scenario: IdentifyResult is serializable

- **WHEN** an `IdentifyResult` is serialized to JSON
- **THEN** the output contains `common_name`, `scientific_name`, and any present optional fields

#### Scenario: JSON schema wraps results in suggestions array

- **WHEN** the identify request is built
- **THEN** the `json_schema` response format SHALL define a root object with `suggestions` (array), `rejected` (boolean), and `rejected_reason` (string or null) properties

#### Scenario: Non-plant photo triggers rejection

- **WHEN** `identify` is called with an image that does not contain a plant
- **THEN** the response SHALL be deserialized into an `IdentifyResponse` with `rejected: true`, a non-null `rejected_reason`, and an empty `suggestions` array

#### Scenario: Identify prompt includes rejection instruction

- **WHEN** `build_identify_prompt` is called
- **THEN** the prompt text SHALL instruct the model to set `rejected` to `true` when the photo does not show a plant
