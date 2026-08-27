## ADDED Requirements

### Requirement: Authentication translations

Each supported locale dictionary SHALL contain matching keys for the login page, authentication-required message, continue-with-provider action, authentication-failed state, provider-unavailable state, logged-out state, Settings Authentication section, Sign out action, and `errorCode.AUTHENTICATION_REQUIRED`. Dynamic provider names SHALL be inserted as text into translated templates.

#### Scenario: Matching locale keys

- **GIVEN** the English, German, and Spanish dictionaries
- **THEN** all authentication keys have identical structures
- **AND** each value is user-facing text in that locale
- **AND** `errorCode.AUTHENTICATION_REQUIRED` exists in every dictionary and matches the backend catalog

#### Scenario: Provider button translation

- **WHEN** the active locale is German or Spanish
- **THEN** the continue action uses the active locale's template
- **AND** includes the configured provider display name

#### Scenario: Generic failure remains translated

- **WHEN** the login page displays authentication-failed or provider-unavailable state
- **THEN** it uses the active locale
- **AND** does not display raw backend or provider diagnostics
