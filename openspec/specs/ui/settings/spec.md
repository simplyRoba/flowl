## Purpose

Settings page with appearance, location management, MQTT status, data stats, and about sections.

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

#### Scenario: Delete location confirmation

- **WHEN** the user clicks delete on a location
- **THEN** a `ModalDialog` is shown in confirm mode with danger variant
- **AND** the dialog message includes the location name and plant count warning if applicable
- **AND** the location is deleted only when the user confirms

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

### Requirement: MQTT Section

The settings page SHALL include an "MQTT" section displaying the MQTT connection status, configuration (fetched from `GET /api/mqtt/status`), and a repair action.

#### Scenario: MQTT connected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "connected"`
- **THEN** the MQTT section shows a "Status" row with a green dot and "Connected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix
- **AND** a "Repair" button is displayed and enabled

#### Scenario: MQTT disconnected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disconnected"`
- **THEN** the MQTT section shows a "Status" row with a muted dot and "Disconnected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix
- **AND** a "Repair" button is displayed but disabled

#### Scenario: MQTT disabled

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disabled"`
- **THEN** the MQTT section shows a "Status" row with "Disabled" text and no indicator dot
- **AND** the "Broker" and "Topic prefix" rows are NOT displayed
- **AND** the "Repair" button is NOT displayed

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/mqtt/status` request fails
- **THEN** the MQTT section is not rendered

#### Scenario: Section ordering

- **WHEN** the settings page loads
- **THEN** the MQTT section appears after "Locations" and before "Data"

#### Scenario: Repair button triggers confirmation

- **GIVEN** the MQTT section is visible and status is "connected"
- **WHEN** the user clicks the "Repair" button
- **THEN** a `ModalDialog` is shown in confirm mode with warning variant
- **AND** the dialog message warns that MQTT topics will be cleared and republished

#### Scenario: Repair confirmed

- **GIVEN** the user confirmed the repair dialog
- **THEN** the button shows a loading state
- **AND** a POST request is sent to `/api/mqtt/repair`

#### Scenario: Repair success feedback

- **GIVEN** the user confirmed repair
- **WHEN** the API responds with HTTP 200 and `{ "cleared": N, "published": M }`
- **THEN** the button returns to its default state
- **AND** an inline message shows the cleared and published counts

#### Scenario: Repair error feedback

- **GIVEN** the user confirmed repair
- **WHEN** the API responds with an error (409 or 503)
- **THEN** the button returns to its default state
- **AND** an inline error message is displayed

### Requirement: Data Section
The settings page Data section SHALL include export and import controls in addition to the existing data statistics.

#### Scenario: Export button
- **WHEN** the settings page loads
- **THEN** the Data section shows an "Export" button
- **AND** clicking it downloads the ZIP export file via `GET /api/data/export`

#### Scenario: Import button
- **WHEN** the settings page loads
- **THEN** the Data section shows an "Import" button
- **AND** clicking it opens a file picker restricted to `.zip` files

#### Scenario: Import confirmation
- **WHEN** the user selects a ZIP file for import
- **THEN** a `ModalDialog` is shown in confirm mode with danger variant
- **AND** the dialog message warns that all existing data and photos will be replaced
- **AND** the dialog message includes the file name

#### Scenario: Import success
- **WHEN** the user confirms the import and the server returns 200
- **THEN** the page reloads the stats to reflect the imported data
- **AND** a success indication is shown

#### Scenario: Import failure
- **WHEN** the user confirms the import and the server returns an error
- **THEN** an error message is displayed
- **AND** existing data remains unchanged

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
