## MODIFIED Requirements

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
- **THEN** a confirmation dialog is shown warning that all existing data and photos will be replaced
- **AND** the dialog shows the file name

#### Scenario: Import success
- **WHEN** the user confirms the import and the server returns 200
- **THEN** the page reloads the stats to reflect the imported data
- **AND** a success indication is shown

#### Scenario: Import failure
- **WHEN** the user confirms the import and the server returns an error
- **THEN** an error message is displayed
- **AND** existing data remains unchanged
