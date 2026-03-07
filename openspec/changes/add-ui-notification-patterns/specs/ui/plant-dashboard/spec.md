## MODIFIED Requirements

### Requirement: Dashboard Attention Card Water Action

Each attention card SHALL include a "Water" button that waters the plant directly from the dashboard and provides visible feedback for the result.

#### Scenario: Water from attention card

- **WHEN** the user clicks the "Water" button on an attention card
- **THEN** a `POST /api/plants/:id/water` request SHALL be sent
- **AND** the plants store SHALL be updated with the new watering data
- **AND** if the plant's status becomes `ok`, it SHALL be removed from the "Needs Attention" section

#### Scenario: Water button loading state

- **WHEN** the user clicks the "Water" button and the request is in progress
- **THEN** the button SHALL indicate a loading state

#### Scenario: Water success feedback

- **WHEN** watering from an attention card succeeds
- **THEN** a visible success acknowledgement SHALL be shown at the time of the action
- **AND** that acknowledgement MAY use the global toast system

#### Scenario: Water failure feedback

- **WHEN** watering from an attention card fails
- **THEN** a visible error acknowledgement SHALL be shown at the time of the action
- **AND** the error SHALL NOT rely solely on distant route-level error text
