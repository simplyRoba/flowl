## MODIFIED Requirements

### Requirement: Plant Detail Watering Section

The plant detail view SHALL display watering status and a "Water now" action.

#### Scenario: Water now success feedback

- **WHEN** the user clicks the "Water now" button and the request succeeds
- **THEN** the view refreshes to show updated watering status
- **AND** no toast notification is required for success

#### Scenario: Water now failure feedback

- **WHEN** the user clicks the "Water now" button and the request fails
- **THEN** a global toast notification is displayed describing the failure

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Delete action success

- **WHEN** the user confirms deletion and the plant is deleted successfully
- **THEN** the app navigates away from the detail page
- **AND** a global toast notification is displayed on the destination page acknowledging the deletion

#### Scenario: Delete care event failure

- **WHEN** the user tries to delete a care event and the request fails
- **THEN** a global toast notification is displayed describing the failure
