## MODIFIED Requirements

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Delete action

- **WHEN** the user clicks the delete button on the detail view
- **THEN** a `ModalDialog` is shown in confirm mode with danger variant
- **AND** the dialog message includes the plant name
- **AND** deletion only proceeds when the user confirms
