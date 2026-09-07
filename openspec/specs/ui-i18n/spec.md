## Purpose

Client-side i18n system providing translations for English, German, and Spanish.

## Requirements

### Requirement: Active locale

The application SHALL use an active locale of `'en'`, `'de'`, or `'es'`, defaulting to `'en'`. The active locale SHALL have a durable browser-local fallback preference that persists across reloads and remains available when the backend cannot be reached.

#### Scenario: Default locale

- **WHEN** the application loads with no stored locale preference and the backend returns the default
- **THEN** the active locale is `'en'`

#### Scenario: Persisted locale restored from backend

- **WHEN** the application loads
- **AND** `GET /api/settings` returns a supported locale
- **AND** the user has not selected a locale since that request began
- **THEN** the active locale is set to the backend value
- **AND** the durable browser-local fallback preference is synchronized to match

#### Scenario: User selection wins over a pending startup response

- **GIVEN** the backend locale request is in progress
- **WHEN** the user selects a supported locale before the response arrives
- **THEN** the response SHALL NOT replace the user's selection
- **AND** the selected locale SHALL remain in the durable browser-local fallback preference across reloads

#### Scenario: Backend unavailable falls back to the browser-local preference

- **WHEN** the application loads
- **AND** `GET /api/settings` fails or the backend is unavailable
- **AND** the durable browser-local fallback preference contains a supported locale (`'en'`, `'de'`, or `'es'`)
- **THEN** the active locale is set to that browser-local value

#### Scenario: Backend and browser-local preference unavailable

- **WHEN** the application loads
- **AND** `GET /api/settings` fails or the backend is unavailable
- **AND** the durable browser-local fallback preference is missing or unsupported
- **THEN** the active locale falls back to `'en'`

#### Scenario: Unsupported persisted locale is rejected

- **WHEN** a locale value from the backend or durable browser-local fallback preference is not `'en'`, `'de'`, or `'es'`
- **THEN** the value SHALL NOT be applied as the active locale
- **AND** the application SHALL use another supported available value or `'en'`

#### Scenario: Locale change persisted to backend

- **WHEN** the locale is changed to a supported value
- **THEN** the new value is sent to the backend via `PUT /api/settings`
- **AND** the durable browser-local fallback preference is updated so the selection persists across reloads and remains usable offline

### Requirement: Translation coverage

The application SHALL provide complete user-facing translations in English, German, and Spanish for the same messages. The required message inventory consists of every heading, label, action, status, validation or error message, and help text explicitly required by canonical UI specifications, together with every error code in the `core-api` Error Code Catalog.

#### Scenario: Matching locale coverage

- **GIVEN** the user-facing messages required by canonical UI specifications
- **THEN** English, German, and Spanish SHALL each provide a translation for every required message

#### Scenario: Error code translations

- **GIVEN** the translations for each supported locale
- **THEN** each locale SHALL provide a user-facing localized message for every error code defined in the backend error catalog

### Requirement: Error Code Resolution

The UI SHALL resolve API error codes to localized strings instead of displaying raw backend messages.

#### Scenario: Known error code displayed in active locale

- **WHEN** an API call fails with a known backend error code
- **AND** the active locale provides a translation for that code
- **THEN** the UI SHALL display the localized translation

#### Scenario: Unknown error code uses fallback

- **WHEN** an API call fails with a backend error code that has no localized translation
- **THEN** the UI SHALL display a generic localized fallback message appropriate to the current context

#### Scenario: Non-API error uses fallback

- **WHEN** an error occurs without a recognized backend error code (e.g., network failure)
- **THEN** the UI SHALL display a generic localized fallback message

#### Scenario: No raw English strings displayed

- **WHEN** the active locale is not English
- **AND** an API error occurs
- **THEN** the displayed error message SHALL be in the active locale, not raw English from the backend

### Requirement: Pluralized results

The UI SHALL use the appropriate localized singular or plural form and substitute the displayed count.

#### Scenario: Singular form

- **WHEN** a one-plant result is displayed in English
- **THEN** the result is `'1 plant'`

#### Scenario: Plural form

- **WHEN** a five-plant result is displayed in English
- **THEN** the result is `'5 plants'`

#### Scenario: Zero uses plural form

- **WHEN** a zero-plant result is displayed in English
- **THEN** the result is `'0 plants'`

### Requirement: Immediate language changes

Visible translated content SHALL update immediately when the active locale changes.

#### Scenario: Translations follow locale

- **GIVEN** the active locale is `'de'`
- **THEN** visible translated content uses German

#### Scenario: Locale change updates translations

- **WHEN** the active locale changes from `'en'` to `'es'`
- **THEN** currently visible translated content updates to Spanish

### Requirement: Authentication translations

Each supported locale SHALL provide user-facing translations for the login page, authentication-required message, continue-with-provider action, authentication-failed state, provider-unavailable state, logged-out state, Settings Authentication section, Sign out action, and the backend `AUTHENTICATION_REQUIRED` error code. Dynamic provider names SHALL be inserted as text into translated messages.

#### Scenario: Matching locale coverage

- **GIVEN** the English, German, and Spanish translations
- **THEN** each locale provides all required authentication messages in user-facing text
- **AND** each locale provides a localized message for the backend `AUTHENTICATION_REQUIRED` error code

#### Scenario: Provider button translation

- **WHEN** the active locale is German or Spanish
- **THEN** the continue action uses the active locale's template
- **AND** includes the configured provider display name

#### Scenario: Generic failure remains translated

- **WHEN** the login page displays authentication-failed or provider-unavailable state
- **THEN** it uses the active locale
- **AND** does not display raw backend or provider diagnostics
