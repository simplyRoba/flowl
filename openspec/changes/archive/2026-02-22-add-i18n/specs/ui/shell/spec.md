## ADDED Requirements

### Requirement: Translated navigation labels

Sidebar nav labels SHALL use translated strings from the locale store instead of hardcoded English text.

#### Scenario: Default English labels

- **GIVEN** the locale is `'en'`
- **WHEN** the sidebar renders
- **THEN** the navigation labels are "Plants", "Log", and "Settings"

#### Scenario: German labels

- **GIVEN** the locale is `'de'`
- **WHEN** the sidebar renders
- **THEN** the navigation labels display the German translations

#### Scenario: Spanish labels

- **GIVEN** the locale is `'es'`
- **WHEN** the sidebar renders
- **THEN** the navigation labels display the Spanish translations

#### Scenario: Widescreen expanded sidebar

- **WHEN** the viewport width is >= 1280px
- **THEN** the expanded sidebar displays translated text labels alongside icons
