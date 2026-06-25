## MODIFIED Requirements

### Requirement: Identify method

The `identify` method SHALL accept a list of images (as byte slices), encode them as base64 data URLs, send them to the configured model using JSON mode (`response_format: { "type": "json_object" }`), and deserialize the response into an `IdentifyResult` containing `common_name`, `scientific_name`, optional `confidence`, optional `summary`, and optional `care_profile`. The `IdentifyResult` and `CareProfile` types SHALL derive both `Serialize` and `Deserialize` so they can be used as HTTP response bodies.

#### Scenario: Single image identification

- **WHEN** `identify` is called with one image
- **THEN** the image is sent as a base64-encoded data URL in the API request
- **AND** the response is deserialized into an `IdentifyResult`

#### Scenario: Multi-image identification

- **WHEN** `identify` is called with multiple images
- **THEN** all images are included in the same API request as separate image content parts

#### Scenario: AI returns incomplete optional fields

- **WHEN** the AI response omits optional fields (confidence, summary, care_profile)
- **THEN** those fields are `None` in the deserialized `IdentifyResult`

#### Scenario: AI returns unparseable response

- **WHEN** the AI response cannot be deserialized into `IdentifyResult`
- **THEN** the method returns an error

#### Scenario: IdentifyResult is serializable

- **WHEN** an `IdentifyResult` is serialized to JSON
- **THEN** the output contains `common_name`, `scientific_name`, and any present optional fields
