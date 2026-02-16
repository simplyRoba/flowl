## Purpose

Settings page with location management section.

## Requirements

### Requirement: Settings Page

The route `/settings` SHALL display a settings page accessible from the sidebar navigation.

#### Scenario: Page loads

- **WHEN** the user navigates to `/settings`
- **THEN** the page displays a "Settings" header

### Requirement: Location Management

The settings page SHALL include a "Locations" section listing all locations with plant counts and delete actions.

#### Scenario: Locations listed

- **WHEN** locations exist
- **THEN** each location is shown with its name and plant count badge (if > 0)
- **AND** a delete button (trash icon) is shown for each location

#### Scenario: Delete location without plants

- **WHEN** the user clicks delete on a location with no plants
- **THEN** a confirmation dialog is shown
- **AND** the location is deleted on confirmation

#### Scenario: Delete location with plants

- **WHEN** the user clicks delete on a location with plants
- **THEN** a confirmation dialog warns about the plant count
- **AND** the location is deleted on confirmation (plants' location_id set to null)

#### Scenario: No locations

- **WHEN** no locations exist
- **THEN** the section shows "No locations yet. Create locations when adding plants."

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
