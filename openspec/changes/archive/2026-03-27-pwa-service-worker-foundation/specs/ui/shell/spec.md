## ADDED Requirements

### Requirement: Offline connectivity indicator

The app shell SHALL display an offline dot badge on the Settings navigation item when the device has no network connectivity. No indicator SHALL be shown when the device is online.

#### Scenario: Dot badge visible when offline

- **WHEN** the device loses network connectivity
- **THEN** a small dot badge SHALL appear on the Settings navigation item

#### Scenario: Dot badge hidden when online

- **WHEN** the device has network connectivity
- **THEN** no dot badge SHALL be displayed on the Settings navigation item

#### Scenario: Initial state reflects connectivity

- **WHEN** the app shell loads
- **THEN** the dot badge SHALL reflect the current connectivity state at load time

#### Scenario: Dot badge updates on connectivity change

- **WHEN** the device transitions between online and offline
- **THEN** the dot badge SHALL appear or disappear without requiring a page reload
