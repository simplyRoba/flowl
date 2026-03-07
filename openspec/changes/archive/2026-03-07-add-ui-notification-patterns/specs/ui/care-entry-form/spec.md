## MODIFIED Requirements

### Requirement: Action buttons

The toolbar-right group SHALL contain cancel and save action buttons.

#### Scenario: Validation failure

- **WHEN** the user attempts to save with invalid or incomplete required input
- **THEN** validation feedback SHALL be displayed inline next to the relevant field or control
- **AND** the request SHALL NOT be sent

#### Scenario: API submission failure

- **WHEN** the save request fails after passing validation
- **THEN** the entered form state SHALL remain populated
- **AND** a global toast notification is displayed describing the failure
