## Purpose

Settings page with location management section.

## Requirements

### Requirement: Settings Page

The route `/settings` SHALL display a settings page accessible from the sidebar navigation.

#### Scenario: Page loads

- **WHEN** the user navigates to `/settings`
- **THEN** the page displays a "Settings" header

### Requirement: Location Management

The settings page SHALL include a "Locations" section listing all locations with plant counts, inline rename, and delete actions.

#### Scenario: Locations listed

- **WHEN** locations exist
- **THEN** each location is shown with its name and plant count badge (if > 0)
- **AND** an edit button (pencil icon) and a delete button (trash icon) are shown for each location

#### Scenario: Edit button enters edit mode

- **WHEN** the user clicks the edit button for a location
- **THEN** the name text is replaced with a text input containing the current name
- **AND** the input is focused with all text selected
- **AND** the edit and delete buttons are replaced with a confirm button (check icon) for that row

#### Scenario: Save renamed location on Enter

- **WHEN** the user is editing a location name and presses Enter
- **THEN** the system calls `updateLocation` with the trimmed new name
- **AND** the row reverts to its default state showing the updated name and action buttons

#### Scenario: Save renamed location on blur

- **WHEN** the user is editing a location name and the input loses focus
- **THEN** the system calls `updateLocation` with the trimmed new name
- **AND** the row reverts to its default state showing the updated name and action buttons

#### Scenario: Name unchanged on confirm

- **WHEN** the user confirms but the trimmed name is identical to the original
- **THEN** the system SHALL NOT call the API
- **AND** the input reverts to static text

#### Scenario: Cancel edit on Escape

- **WHEN** the user is editing a location name and presses Escape
- **THEN** the input reverts to static text showing the original name
- **AND** no API call is made

#### Scenario: Reject empty location name

- **WHEN** the user empties the location name input and presses Enter or blurs
- **THEN** the input reverts to the original name
- **AND** no API call is made

#### Scenario: Rename conflicts with existing location

- **WHEN** the user renames a location to a name that already exists
- **THEN** the backend returns 409
- **AND** the input remains in edit mode with the attempted name
- **AND** an inline error message is displayed below the input

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

### Requirement: About Section

The settings page SHALL include an "About" section displaying application metadata fetched from `GET /api/info`.

#### Scenario: About section displays version

- **WHEN** the settings page loads
- **THEN** the About section shows a "Version" row with the app version from the API

#### Scenario: About section displays source link

- **WHEN** the settings page loads
- **THEN** the About section shows a "Source" row with the repository URL as a clickable link
- **AND** the link text displays the URL without the `https://` prefix

#### Scenario: About section displays license

- **WHEN** the settings page loads
- **THEN** the About section shows a "License" row with the license identifier from the API

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/info` request fails
- **THEN** the About section is not rendered
