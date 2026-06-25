## ADDED Requirements

### Requirement: Apply persisted theme preference
The UI shell SHALL apply the stored theme preference across all screens.

#### Scenario: Stored light preference
- **GIVEN** the stored theme preference is `light`
- **WHEN** the UI shell loads
- **THEN** the UI renders with light theme tokens

#### Scenario: Stored dark preference
- **GIVEN** the stored theme preference is `dark`
- **WHEN** the UI shell loads
- **THEN** the UI renders with dark theme tokens

### Requirement: System theme preference
The UI shell SHALL follow the system color scheme when the theme preference is `system`.

#### Scenario: System preference is dark
- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is dark
- **WHEN** the UI shell loads
- **THEN** the UI renders with dark theme tokens

#### Scenario: System preference is light
- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is light
- **WHEN** the UI shell loads
- **THEN** the UI renders with light theme tokens

#### Scenario: System preference changes
- **GIVEN** the stored theme preference is `system`
- **WHEN** the system color scheme changes
- **THEN** the UI updates to the new theme tokens without a reload
