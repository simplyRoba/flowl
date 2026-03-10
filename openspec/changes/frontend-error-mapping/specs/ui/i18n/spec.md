## MODIFIED Requirements

### Requirement: Translation dictionaries

Each supported locale SHALL have a TypeScript translation object with identical keys.

#### Scenario: Dictionary structure

- **GIVEN** the English, German, and Spanish translation dictionaries
- **THEN** all three dictionaries SHALL have identical key structures
- **AND** keys are organized in shallow nested groups (e.g., `nav`, `dashboard`, `plant`, `status`, `settings`, `care`, `form`, `identify`, `dialog`, `chat`, `errorCode`)

#### Scenario: English as canonical type

- **GIVEN** the translation dictionaries
- **THEN** the English dictionary SHALL serve as the canonical TypeScript type definition
- **AND** the German and Spanish dictionaries SHALL satisfy the same type

#### Scenario: Error code translations

- **GIVEN** the `errorCode` group in each translation dictionary
- **THEN** each dictionary SHALL contain a key for every error code defined in the backend error catalog
- **AND** values SHALL be user-facing translated strings appropriate for the locale

## ADDED Requirements

### Requirement: Error Code Resolution

The UI SHALL resolve API error codes to localized strings instead of displaying raw backend messages.

#### Scenario: Known error code displayed in active locale

- **WHEN** an API call fails with an `ApiError` carrying a known `code`
- **AND** the active locale has a translation for that code in `errorCode`
- **THEN** the UI SHALL display the localized translation

#### Scenario: Unknown error code uses fallback

- **WHEN** an API call fails with an `ApiError` carrying a code not present in `errorCode`
- **THEN** the UI SHALL display a generic localized fallback message from the store's context-specific error key

#### Scenario: Non-API error uses fallback

- **WHEN** an error occurs that is not an `ApiError` (e.g., network failure)
- **THEN** the UI SHALL display a generic localized fallback message

#### Scenario: No raw English strings displayed

- **WHEN** the active locale is not English
- **AND** an API error occurs
- **THEN** the displayed error message SHALL be in the active locale, not raw English from the backend
