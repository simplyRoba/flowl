## Purpose

Self-contained care entry form component with toolbar-absorbed layout, compound photo/date controls, grouped flex wrapping, and encapsulated state.

## Requirements

### Requirement: CareEntryForm component

The `CareEntryForm` component (`$lib/components/CareEntryForm.svelte`) SHALL be a self-contained form for logging care events on a plant. It SHALL accept a `plantId` prop and emit `onsubmit` and `oncancel` callbacks.

#### Scenario: Component renders with event type chips

- **WHEN** the component is rendered
- **THEN** it SHALL display a row of event type chips: fertilized, repotted, pruned, custom
- **AND** each chip SHALL use the corresponding lucide icon and translation label
- **AND** no chip SHALL be selected by default

#### Scenario: Selecting an event type

- **WHEN** the user clicks a type chip
- **THEN** that chip SHALL become active (`.chip-solid.active`)
- **AND** clicking a different chip SHALL switch the selection

#### Scenario: Notes textarea

- **WHEN** the component is rendered
- **THEN** a textarea SHALL be displayed below the type chips with placeholder text from translations
- **AND** the textarea SHALL be 2 rows by default

### Requirement: Toolbar layout

The form SHALL display a toolbar row below the textarea containing tool buttons (left group) and action buttons (right group).

#### Scenario: Toolbar structure

- **WHEN** the component is rendered
- **THEN** the toolbar SHALL be a flex container with `flex-wrap: wrap`
- **AND** it SHALL contain two inner groups: toolbar-left and toolbar-right
- **AND** toolbar-right SHALL use `margin-left: auto` to align right
- **AND** each group SHALL NOT break internally when wrapping

#### Scenario: Toolbar wrapping on narrow viewports

- **WHEN** the viewport is narrow enough that both groups cannot fit on one row
- **THEN** toolbar-right SHALL wrap to a new row
- **AND** the cancel and save buttons SHALL remain together on the same row

### Requirement: Photo tool button

The toolbar-left group SHALL contain a photo tool button that morphs between inactive and active states.

#### Scenario: Photo button inactive

- **WHEN** no photo is attached
- **THEN** the toolbar SHALL show a ghost-style icon button with the Camera icon
- **AND** clicking it SHALL open the file picker (via hidden `<input type="file">`)
- **AND** accepted types SHALL be `image/jpeg, image/png, image/webp`

#### Scenario: Photo button active (compound group)

- **WHEN** a photo is attached
- **THEN** the camera icon button SHALL be replaced by a compound group: `[thumbnail | x]`
- **AND** the compound group SHALL have a shared border with `border-radius: var(--radius-btn)`
- **AND** the thumbnail SHALL show the selected image as `object-fit: cover` using a local object URL (not a server thumbnail, since the photo has not been uploaded yet)
- **AND** the dismiss button SHALL use XIcon at size 12
- **AND** the compound group SHALL NOT show a camera icon (no redundant icon)

#### Scenario: Dismissing a photo

- **WHEN** the user clicks the dismiss button on the photo compound group
- **THEN** the photo SHALL be cleared
- **AND** the compound group SHALL revert to the inactive camera icon button
- **AND** the preview object URL SHALL be revoked

### Requirement: Backdate tool button

The toolbar-left group SHALL contain a backdate tool button that morphs between inactive and active states.

#### Scenario: Backdate button inactive

- **WHEN** backdate is not active
- **THEN** the toolbar SHALL show a ghost-style icon button with the CalendarClock icon
- **AND** clicking it SHALL activate backdate mode

#### Scenario: Backdate button active (compound group)

- **WHEN** backdate is active
- **THEN** the calendar icon button SHALL be replaced by a compound group: `[datetime-local input | ✕]`
- **AND** the input SHALL default to the current date/time
- **AND** the input `max` SHALL be the current date/time
- **AND** the compound group SHALL NOT show a calendar icon (no redundant icon)

#### Scenario: Dismissing backdate

- **WHEN** the user clicks the dismiss button on the backdate compound group
- **THEN** backdate mode SHALL deactivate
- **AND** the compound group SHALL revert to the inactive calendar icon button

### Requirement: Action buttons

The toolbar-right group SHALL contain cancel and save action buttons.

#### Scenario: Save button

- **WHEN** the form is rendered
- **THEN** a primary save button SHALL be displayed
- **AND** it SHALL be disabled until an event type is selected
- **AND** clicking it SHALL submit the care event via `POST /api/plants/{id}/care-logs`
- **AND** if a photo is attached, it SHALL be uploaded after event creation
- **AND** on success, the `onsubmit` callback SHALL be called

#### Scenario: Cancel button

- **WHEN** the user clicks cancel
- **THEN** all form state SHALL be reset (event type, notes, photo, backdate)
- **AND** the `oncancel` callback SHALL be called

#### Scenario: Submitting state

- **WHEN** a submission is in progress
- **THEN** the save button SHALL show the saving translation text
- **AND** the save button SHALL be disabled

#### Scenario: Validation failure

- **WHEN** the user attempts to save with invalid or incomplete required input
- **THEN** validation feedback SHALL be displayed inline next to the relevant field or control
- **AND** the request SHALL NOT be sent

#### Scenario: API submission failure

- **WHEN** the save request fails after passing validation
- **THEN** the entered form state SHALL remain populated
- **AND** a global toast notification is displayed describing the failure
