## ADDED Requirements

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
- **AND** the preference is saved for future sessions
