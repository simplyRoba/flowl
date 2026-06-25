## MODIFIED Requirements

### Requirement: Identify error state

The identify section SHALL display an error state when the identification request fails. When the error code is `AI_IDENTIFY_NOT_A_PLANT`, the error message SHALL display the localized "not a plant" message from the `errorCode` i18n map. The error state SHALL include a "Retry" button regardless of the error code.

#### Scenario: Error displayed

- **WHEN** the `POST /api/ai/identify` request fails (network error, 500, 503, or other error)
- **THEN** the identify section SHALL display an error message and a "Retry" button

#### Scenario: Not-a-plant error displayed

- **WHEN** the `POST /api/ai/identify` request returns 422 with code `AI_IDENTIFY_NOT_A_PLANT`
- **THEN** the identify section SHALL display the localized "not a plant" message from the `errorCode` i18n map
- **AND** a "Retry" button SHALL be displayed

#### Scenario: Retry after error

- **WHEN** the user clicks "Retry"
- **THEN** the identification request SHALL be re-submitted with the same photos
- **AND** the section SHALL transition to the loading state

## ADDED Requirements

### Requirement: Not-a-plant error code in i18n

The `errorCode` map in all supported locales (en, de, es) SHALL include an `AI_IDENTIFY_NOT_A_PLANT` entry with a localized message indicating the photo does not appear to contain a plant.

#### Scenario: English locale

- **WHEN** the locale is `en`
- **THEN** the `AI_IDENTIFY_NOT_A_PLANT` error code SHALL resolve to a message like "The photo does not appear to contain a plant"

#### Scenario: German locale

- **WHEN** the locale is `de`
- **THEN** the `AI_IDENTIFY_NOT_A_PLANT` error code SHALL resolve to a German translation of the message

#### Scenario: Spanish locale

- **WHEN** the locale is `es`
- **THEN** the `AI_IDENTIFY_NOT_A_PLANT` error code SHALL resolve to a Spanish translation of the message
