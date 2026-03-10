## Purpose

Client-side i18n system providing translations for English, German, and Spanish.

## Requirements

### Requirement: Locale store

A `locale` writable store SHALL hold the active locale (`'en' | 'de' | 'es'`), defaulting to `'en'`.

#### Scenario: Default locale

- **WHEN** the application loads with no stored locale preference and the backend returns the default
- **THEN** the locale store is set to `'en'`

#### Scenario: Persisted locale restored from backend

- **WHEN** the application loads
- **AND** `GET /api/settings` returns a valid locale
- **THEN** the locale store is set to the backend value
- **AND** `localStorage` key `flowl.locale` is updated to match

#### Scenario: Backend unavailable falls back to localStorage

- **WHEN** the application loads
- **AND** `GET /api/settings` fails
- **AND** `localStorage` key `flowl.locale` contains a valid locale (`'en'`, `'de'`, or `'es'`)
- **THEN** the locale store is set to the `localStorage` value

#### Scenario: Both backend and localStorage unavailable

- **WHEN** the application loads
- **AND** `GET /api/settings` fails
- **AND** `localStorage` key `flowl.locale` is missing or invalid
- **THEN** the locale store falls back to `'en'`

#### Scenario: Locale change persisted to backend

- **WHEN** the locale is changed
- **THEN** the new value is sent to the backend via `PUT /api/settings`
- **AND** the new value is written to `localStorage` key `flowl.locale` as a fallback cache

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

### Requirement: Plural helper

A `plural(forms: {one: string, other: string}, n: number)` function SHALL return the correct plural form with count substitution.

#### Scenario: Singular form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 1)` is called
- **THEN** the result is `'1 plant'`

#### Scenario: Plural form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 5)` is called
- **THEN** the result is `'5 plants'`

#### Scenario: Zero uses plural form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 0)` is called
- **THEN** the result is `'0 plants'`

### Requirement: Reactive translations

A derived store `translations` SHALL resolve to the translation object for the current locale.

#### Scenario: Translations follow locale

- **GIVEN** the locale store is set to `'de'`
- **THEN** the `translations` store resolves to the German translation dictionary

#### Scenario: Locale change updates translations

- **WHEN** the locale store changes from `'en'` to `'es'`
- **THEN** the `translations` store reactively updates to the Spanish translation dictionary

#### Scenario: Component access pattern

- **GIVEN** a Svelte component subscribes to the `translations` store
- **THEN** translated strings are accessed via `$translations.group.key`
