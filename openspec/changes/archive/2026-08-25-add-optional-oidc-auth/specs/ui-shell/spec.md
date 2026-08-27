## ADDED Requirements

### Requirement: Public login shell isolation

The public `/login` route SHALL render outside Flowl's protected application shell. Existing canonical shell layout, navigation, widths, scrolling, theme, and pull-to-refresh behavior SHALL remain unchanged for normal application routes.

#### Scenario: Login route omits protected shell

- **WHEN** `/login` is loaded
- **THEN** the branded login content is visible without the application sidebar or bottom navigation
- **AND** the root layout does not request `/api/settings` or other protected application data
- **AND** it does not initialize the network monitor, service-worker update UI, or pull-to-refresh behavior

#### Scenario: Normal routes retain canonical shell

- **WHEN** a normal authenticated application route is loaded
- **THEN** the existing canonical application shell and navigation behavior remains in effect

#### Scenario: Login route transition is reactively stable

- **WHEN** the SPA transitions to or from `/login`
- **THEN** the layout SHALL NOT enter a reactive update loop
- **AND** no effect SHALL both depend on and mutate the same authentication or pull-to-refresh state
