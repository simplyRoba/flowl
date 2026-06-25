## ADDED Requirements

### Requirement: Care journal offline message

The global care journal page SHALL display an offline-specific message instead of a generic error when the network is unavailable.

#### Scenario: Offline message shown when fetch fails offline

- **WHEN** the care journal page attempts to load events
- **AND** the fetch fails
- **AND** `navigator.onLine` is `false`
- **THEN** the page SHALL display a translated offline message instead of the generic load error text

#### Scenario: Generic error shown when fetch fails online

- **WHEN** the care journal page attempts to load events
- **AND** the fetch fails
- **AND** `navigator.onLine` is `true`
- **THEN** the page SHALL display the existing generic error text from `resolveError()`

#### Scenario: Skeleton shown before offline determination

- **WHEN** the care journal page is loading events
- **THEN** the existing skeleton shimmer lines SHALL be displayed while the fetch is in progress
- **AND** the offline message SHALL only appear after the fetch fails
