## Purpose

Themed confirmation prompts and alert messages using the HTML `<dialog>` element.

## Requirements

### Requirement: Confirmation and alert dialogs
The system SHALL provide themed confirmation prompts and alert messages using the HTML `<dialog>` element.

#### Scenario: Confirmation prompt displays two buttons
- **WHEN** a confirmation prompt is opened
- **THEN** the dialog displays a title, message, a "Cancel" button, and an action button with an action-specific label

#### Scenario: Alert displays one button
- **WHEN** an alert is opened
- **THEN** the dialog displays a title, message, and a single "OK" button

#### Scenario: Destructive dialog styling
- **WHEN** a destructive confirmation prompt or alert is opened
- **THEN** its action or OK button uses danger styling (red)

#### Scenario: Warning dialog styling
- **WHEN** a warning confirmation prompt or alert is opened
- **THEN** its action or OK button uses primary styling

#### Scenario: Confirmation accepted
- **WHEN** the user clicks the action button in a confirmation prompt
- **THEN** the requested action SHALL proceed
- **AND** the dialog SHALL close

#### Scenario: Confirmation cancelled
- **WHEN** the user clicks the cancel button in a confirmation prompt
- **THEN** the requested action SHALL NOT proceed
- **AND** the dialog SHALL close

#### Scenario: Alert acknowledged
- **WHEN** the user clicks the OK button in an alert
- **THEN** the alert SHALL close

#### Scenario: Escape key in confirmation prompt
- **WHEN** the user presses Escape while a confirmation prompt is open
- **THEN** the requested action SHALL NOT proceed
- **AND** the dialog SHALL close

#### Scenario: Escape key in alert
- **WHEN** the user presses Escape while an alert is open
- **THEN** the alert SHALL close

#### Scenario: Backdrop click in confirmation prompt
- **WHEN** the user clicks the backdrop behind a confirmation prompt
- **THEN** the requested action SHALL NOT proceed
- **AND** the dialog SHALL close

#### Scenario: Backdrop click in alert
- **WHEN** the user clicks the backdrop behind an alert
- **THEN** the dialog remains open

#### Scenario: Dialog visibility
- **WHEN** a confirmation prompt or alert is opened
- **THEN** the dialog opens via `showModal()`
- **AND** when it is closed
- **THEN** the dialog closes via `close()`
