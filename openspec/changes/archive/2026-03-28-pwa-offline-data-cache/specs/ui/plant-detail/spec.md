## ADDED Requirements

### Requirement: Plant detail offline browsing

The plant detail page SHALL be viewable offline using cached API data when the network is unavailable.

#### Scenario: Plant detail loads from cache when offline

- **WHEN** the user navigates to `/plants/{id}` while offline
- **AND** cached responses exist for `/api/plants/{id}` and `/api/plants/{id}/care`
- **THEN** the plant detail page SHALL display the plant data and care event timeline using cached data

#### Scenario: Plant detail with no cache and no network

- **WHEN** the user navigates to `/plants/{id}` while offline
- **AND** no cached response exists for `/api/plants/{id}`
- **THEN** the plant detail page SHALL display the existing error state

#### Scenario: Care events with no cache and no network

- **WHEN** the plant detail loads from cache while offline
- **AND** no cached response exists for `/api/plants/{id}/care`
- **THEN** the care journal section SHALL display the existing skeleton followed by the care error state

### Requirement: Plant detail mutation controls disabled when offline

Mutation actions on the plant detail page SHALL be disabled when the device is offline.

#### Scenario: Water now button disabled when offline

- **WHEN** the plant detail page is rendered while offline
- **THEN** the "Water now" button SHALL be visually disabled
- **AND** clicking it SHALL NOT send a request

#### Scenario: Add log entry button disabled when offline

- **WHEN** the plant detail page is rendered while offline
- **THEN** the "Add log entry" button SHALL be visually disabled
- **AND** clicking it SHALL NOT open the care entry form

#### Scenario: Edit and delete actions disabled when offline

- **WHEN** the plant detail page is rendered while offline
- **THEN** the edit and delete action buttons SHALL be visually disabled
- **AND** clicking them SHALL NOT navigate or open a dialog

#### Scenario: Mutation controls re-enabled when back online

- **WHEN** the device transitions from offline to online
- **THEN** all mutation controls on the plant detail page SHALL become enabled again
