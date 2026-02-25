## MODIFIED Requirements

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
