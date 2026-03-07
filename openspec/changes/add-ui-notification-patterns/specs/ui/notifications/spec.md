## ADDED Requirements

### Requirement: Feedback Placement Taxonomy

The UI SHALL choose feedback placement based on whether the user must correct nearby state, whether the source context remains visible, and whether the route can continue functioning.

#### Scenario: Field correction uses inline field feedback

- **WHEN** an error belongs to a specific input or control
- **AND** the user must edit that nearby control to recover
- **THEN** the error SHALL be displayed inline adjacent to that control
- **AND** a toast SHALL NOT be the primary feedback surface

#### Scenario: Contextual operation uses inline section feedback

- **WHEN** an action result is best understood inside its owning section
- **AND** the feedback may include detail, retry, or multiple values
- **THEN** the feedback SHALL be displayed inline within that section

#### Scenario: Out-of-context acknowledgement uses toast feedback

- **WHEN** an action completes or fails and no nearby corrective input is required
- **AND** the trigger may disappear, navigate away, or be offscreen
- **THEN** the UI MAY display a global toast notification
- **AND** the message SHALL remain understandable without nearby UI context

#### Scenario: Route load failure uses persistent page-level feedback

- **WHEN** a route fails to load its primary data
- **THEN** the route SHALL display a persistent page-level error state
- **AND** a toast SHALL NOT be the sole feedback surface

### Requirement: Global Toast Host

The UI SHALL provide a global toast host that can display notifications from any route.

#### Scenario: Toasts available from any route

- **WHEN** a route or shared component emits a toast notification
- **THEN** the notification SHALL render in a single global host mounted at the application-shell level

#### Scenario: Stack limit

- **WHEN** multiple toast notifications are active
- **THEN** the host SHALL display at most 3 visible toasts at once
- **AND** additional toasts SHALL be queued or otherwise prevented from overflowing the viewport

### Requirement: Responsive Toast Placement

The toast host SHALL adapt to viewport constraints and MUST NOT compete with the mobile bottom navigation area.

#### Scenario: Desktop and tablet placement

- **WHEN** the viewport width is greater than 768px
- **THEN** the visible toast stack SHALL be anchored to the bottom-right of the viewport

#### Scenario: Mobile placement

- **WHEN** the viewport width is less than or equal to 768px
- **THEN** the visible toast stack SHALL be anchored near the top of the viewport below the safe-area inset
- **AND** it SHALL NOT be anchored above the fixed bottom nav area

### Requirement: Toast Severity Behavior

The toast system SHALL vary dismissal behavior by severity.

#### Scenario: Success and info dismiss automatically

- **WHEN** a `success` or `info` toast is shown
- **THEN** it SHALL auto-dismiss after a short timeout

#### Scenario: Warning remains visible longer

- **WHEN** a `warning` toast is shown
- **THEN** it SHALL remain visible longer than a `success` toast
- **AND** it SHALL be manually dismissible

#### Scenario: Error requires dismissal

- **WHEN** an `error` toast is shown
- **THEN** it SHALL be manually dismissible

### Requirement: Toast Accessibility

The toast system SHALL announce notifications in a way that matches their urgency.

#### Scenario: Non-error announcement politeness

- **WHEN** a `success` or `info` toast is shown
- **THEN** it SHALL use polite live-region semantics

#### Scenario: Error announcement urgency

- **WHEN** an `error` toast is shown
- **THEN** it SHALL use assertive live-region semantics
