## ADDED Requirements

### Requirement: Settings offline message

The settings page SHALL display an offline-specific message when the network is unavailable and data sections cannot load.

#### Scenario: Offline message shown when offline

- **WHEN** the settings page loads
- **AND** `navigator.onLine` is `false`
- **AND** data fetches for settings sections fail
- **THEN** the page SHALL display a translated offline message in place of the sections that failed to load

#### Scenario: Appearance and language sections remain functional offline

- **WHEN** the settings page loads while offline
- **THEN** the Appearance theme selector and Language selector SHALL remain functional
- **AND** theme and language changes SHALL be applied locally (stored in `localStorage`)
- **AND** the server-side persistence (`PUT /api/settings`) MAY fail silently

#### Scenario: Normal errors shown when online

- **WHEN** the settings page loads while online
- **AND** a data fetch fails
- **THEN** the existing behavior SHALL apply (sections not rendered on failure)
