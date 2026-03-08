## ADDED Requirements

### Requirement: Pull-to-refresh in standalone PWA mode

The app SHALL provide a custom pull-to-refresh gesture on allowlisted browse routes when running in standalone PWA mode on touch devices.

#### Scenario: Pull-to-refresh available on dashboard
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on care journal
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/care-journal`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on settings
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/settings`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

#### Scenario: Pull-to-refresh available on plant detail
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/42`
- **AND** the document is scrolled to the top
- **THEN** pulling down from the top SHALL arm a refresh gesture

### Requirement: Pull-to-refresh route exclusions

The app SHALL NOT provide the custom pull-to-refresh gesture on non-allowlisted routes.

#### Scenario: New plant route excluded
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/new`
- **THEN** pulling down SHALL NOT arm the custom refresh gesture

#### Scenario: Edit plant route excluded
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on `/plants/42/edit`
- **THEN** pulling down SHALL NOT arm the custom refresh gesture

### Requirement: Pull-to-refresh reload behavior

Once armed on an allowlisted route, the gesture SHALL trigger a full reload of the current route when the user releases beyond the refresh threshold.

#### Scenario: Release beyond threshold reloads page
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on an allowlisted route
- **AND** the document is scrolled to the top
- **AND** the user pulls beyond the refresh threshold and releases
- **THEN** the app SHALL perform a full reload of the current route

#### Scenario: Release before threshold does not reload page
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user is on an allowlisted route
- **AND** the document is scrolled to the top
- **AND** the user releases before reaching the refresh threshold
- **THEN** the app SHALL cancel the gesture without reloading the route

### Requirement: Pull-to-refresh feedback and safety gates

The app SHALL provide visible feedback while the gesture is active and SHALL suppress the gesture when the browsing context is not safe for refresh.

#### Scenario: Feedback shown during pull
- **WHEN** the app is running in standalone PWA mode on a touch device
- **AND** the user pulls down from the top on an allowlisted route
- **THEN** the app SHALL display a visible pull-to-refresh indicator

#### Scenario: Gesture ignored away from top of page
- **WHEN** the user is on an allowlisted route
- **AND** the document is not scrolled to the top
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm

#### Scenario: Gesture suppressed while transient overlay is open
- **WHEN** the user is on `/plants/42`
- **AND** a transient overlay such as a modal dialog, lightbox, chat drawer, or inline care entry flow is open
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm

#### Scenario: Gesture unavailable outside standalone mode
- **WHEN** the app is running in a normal browser tab instead of standalone PWA mode
- **THEN** the custom pull-to-refresh gesture SHALL NOT arm
