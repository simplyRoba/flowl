## ADDED Requirements

### Requirement: Dashboard offline browsing

The dashboard SHALL be browsable offline using cached API data when the network is unavailable.

#### Scenario: Dashboard loads from cache when offline

- **WHEN** the user navigates to `/` while offline
- **AND** cached responses exist for `/api/plants`
- **THEN** the dashboard SHALL display the plant grid using the cached data

#### Scenario: Dashboard with no cache and no network

- **WHEN** the user navigates to `/` while offline
- **AND** no cached response exists for `/api/plants`
- **THEN** the dashboard SHALL display the existing error state

### Requirement: Dashboard mutation controls disabled when offline

Mutation actions on the dashboard SHALL be disabled when the device is offline.

#### Scenario: Water button disabled when offline

- **WHEN** the dashboard is rendered while offline
- **AND** a plant appears in the "Needs Attention" section
- **THEN** the "Water" button on the attention card SHALL be visually disabled
- **AND** clicking it SHALL NOT send a request

#### Scenario: Water button re-enabled when back online

- **WHEN** the device transitions from offline to online
- **THEN** the "Water" buttons on attention cards SHALL become enabled again
