## MODIFIED Requirements

### Requirement: Appearance theme selector

The settings page SHALL include an Appearance section that lets the user choose Light, Dark, or System theme.

#### Scenario: Settings page shows theme options

- **WHEN** the user navigates to `/settings`
- **THEN** the page displays an Appearance section
- **AND** the theme selector shows Light, Dark, and System options

#### Scenario: Theme option selection

- **GIVEN** the settings page is visible
- **WHEN** the user selects a theme option
- **THEN** the selected option is visually indicated as active
- **AND** the preference is persisted to the backend via `PUT /api/settings`
- **AND** the preference is also written to `localStorage` as a fallback cache

#### Scenario: Theme initialised from backend

- **WHEN** the application loads
- **THEN** the theme preference is fetched from `GET /api/settings`
- **AND** the theme store is seeded with the backend value
- **AND** `localStorage` is updated to match

#### Scenario: Backend unavailable on init

- **WHEN** the application loads and `GET /api/settings` fails
- **THEN** the theme store falls back to the `localStorage` value
- **AND** if `localStorage` is also empty, the default `'system'` is used
