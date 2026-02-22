## ADDED Requirements

### Requirement: Language selector

The settings page SHALL include a "Language" section (after Appearance) with a pill toggle showing "English", "Deutsch", "Español".

#### Scenario: Settings page shows language options

- **WHEN** the user navigates to `/settings`
- **THEN** the page displays a "Language" section after the Appearance section
- **AND** the language selector shows English, Deutsch, and Español options

#### Scenario: Language option selection

- **GIVEN** the settings page is visible
- **WHEN** the user selects a language option
- **THEN** the selected option is visually indicated as active
- **AND** the locale store is updated immediately
- **AND** the preference is persisted to `localStorage`

#### Scenario: Reactive UI update

- **GIVEN** the settings page is visible
- **WHEN** the user selects a different language
- **THEN** all visible UI text on the page updates reactively to the selected language
