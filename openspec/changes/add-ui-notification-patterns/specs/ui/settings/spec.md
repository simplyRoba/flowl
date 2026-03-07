## MODIFIED Requirements

### Requirement: MQTT Section

The settings page SHALL include an "MQTT" section displaying the MQTT connection status, configuration, and a repair action.

#### Scenario: Repair success feedback

- **GIVEN** the user confirmed repair
- **WHEN** the API responds with HTTP 200 and `{ "cleared": N, "published": M }`
- **THEN** the button returns to its default state
- **AND** a global toast notification is displayed acknowledging the repair result

#### Scenario: Repair error feedback

- **GIVEN** the user confirmed repair
- **WHEN** the API responds with an error
- **THEN** the button returns to its default state
- **AND** a global toast notification is displayed describing the failure

### Requirement: Data Section

The settings page Data section SHALL include export and import controls in addition to the existing data statistics.

#### Scenario: Import success

- **WHEN** the user confirms the import and the server returns 200
- **THEN** the page reloads the stats to reflect the imported data
- **AND** a global toast notification is displayed acknowledging success

#### Scenario: Import failure

- **WHEN** the user confirms the import and the server returns an error
- **THEN** existing data remains unchanged
- **AND** a global toast notification is displayed describing the failure

#### Scenario: Export success

- **WHEN** the user clicks export and the browser starts the file download normally
- **THEN** the UI MAY remain silent without an additional success toast

#### Scenario: Export detectable failure

- **WHEN** the export flow can detect a failure before or during download handoff
- **THEN** a global toast notification is displayed describing the failure
