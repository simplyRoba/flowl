## ADDED Requirements

### Requirement: ModalDialog component
The system SHALL provide a reusable `ModalDialog` component using the HTML `<dialog>` element for themed confirmation prompts and alert messages.

#### Scenario: Confirm mode renders two buttons
- **WHEN** a `ModalDialog` is opened with `mode="confirm"`
- **THEN** the dialog displays a title, message, a "Cancel" button, and a confirm button with the `confirmLabel` text

#### Scenario: Alert mode renders one button
- **WHEN** a `ModalDialog` is opened with `mode="alert"`
- **THEN** the dialog displays a title, message, and a single "OK" button

#### Scenario: Danger variant styling
- **WHEN** a `ModalDialog` is opened with `variant="danger"`
- **THEN** the confirm/OK button uses danger styling (red)

#### Scenario: Warning variant styling
- **WHEN** a `ModalDialog` is opened with `variant="warning"`
- **THEN** the confirm/OK button uses primary styling

#### Scenario: Confirm callback
- **WHEN** the user clicks the confirm button in confirm mode
- **THEN** the `onconfirm` callback is fired
- **AND** the dialog closes

#### Scenario: Cancel callback
- **WHEN** the user clicks the cancel button in confirm mode
- **THEN** the `oncancel` callback is fired
- **AND** the dialog closes

#### Scenario: Alert close callback
- **WHEN** the user clicks the OK button in alert mode
- **THEN** the `onclose` callback is fired
- **AND** the dialog closes

#### Scenario: Escape key in confirm mode
- **WHEN** the user presses Escape while a confirm-mode dialog is open
- **THEN** the `oncancel` callback is fired
- **AND** the dialog closes

#### Scenario: Escape key in alert mode
- **WHEN** the user presses Escape while an alert-mode dialog is open
- **THEN** the `onclose` callback is fired
- **AND** the dialog closes

#### Scenario: Backdrop click in confirm mode
- **WHEN** the user clicks the backdrop behind a confirm-mode dialog
- **THEN** the `oncancel` callback is fired
- **AND** the dialog closes

#### Scenario: Backdrop click in alert mode
- **WHEN** the user clicks the backdrop behind an alert-mode dialog
- **THEN** the dialog remains open

#### Scenario: Open prop controls visibility
- **WHEN** the `open` prop changes from `false` to `true`
- **THEN** the dialog opens via `showModal()`
- **AND** when `open` changes from `true` to `false`
- **THEN** the dialog closes via `close()`
