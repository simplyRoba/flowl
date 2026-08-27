## ADDED Requirements

### Requirement: Authentication Settings section

When built-in authentication is enabled, the Settings page SHALL show a translated Authentication section containing a translated `Sign out` action. The section SHALL NOT display identity claims, provider tokens, roles, groups, or account-management controls and SHALL be hidden when authentication is disabled.

#### Scenario: Authentication section enabled

- **WHEN** the Settings page loads and `/auth/config` reports `enabled: true`
- **THEN** a translated Authentication section is displayed
- **AND** it contains a Sign out action

#### Scenario: Sign out action

- **WHEN** the user activates Sign out
- **THEN** the frontend performs the explicit protected-cache purge and local POST logout flow

#### Scenario: Authentication section disabled

- **WHEN** `/auth/config` reports `enabled: false`
- **THEN** the Authentication section and Sign out action are not rendered

#### Scenario: No identity management fields

- **WHEN** the Authentication section is visible
- **THEN** it contains no username, password, registration, profile, role, group, permission, or provider-session controls
