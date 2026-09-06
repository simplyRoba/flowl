## Purpose

Care entry form with an integrated toolbar, combined photo/date controls, and responsive action grouping.

## Requirements

### Requirement: Care entry form

The system SHALL provide a form for creating or editing care events for a plant.

#### Scenario: Add form displays event type chips

- **WHEN** the care entry form is opened to add an event
- **THEN** it SHALL display a row of event type chips: watered, fertilized, repotted, pruned, custom
- **AND** each chip SHALL use a recognizable event-type visual marker and translated label
- **AND** no chip SHALL be selected by default

#### Scenario: Edit form displays existing type selected

- **WHEN** the care entry form is opened to edit an existing care event
- **THEN** it SHALL display the same event type chips
- **AND** the existing event type chip SHALL be selected

#### Scenario: Selecting an event type

- **WHEN** the user clicks a type chip
- **THEN** that chip SHALL show a distinguishable active state
- **AND** clicking a different chip SHALL switch the selection

#### Scenario: Notes textarea

- **WHEN** the care entry form is opened
- **THEN** a textarea SHALL be displayed below the type chips with placeholder text from translations
- **AND** the textarea SHALL be 2 rows by default
- **AND** in edit mode it SHALL contain the existing notes when present

### Requirement: Toolbar layout

The form SHALL display a toolbar row below the textarea containing tool buttons (left group) and action buttons (right group).

#### Scenario: Toolbar structure

- **WHEN** the care entry form is opened
- **THEN** the toolbar SHALL contain a tool-controls group and an action-controls group
- **AND** the action-controls group SHALL appear at the trailing edge when both groups fit on one row
- **AND** each group SHALL remain intact when the toolbar reflows

#### Scenario: Toolbar wrapping on narrow viewports

- **WHEN** the viewport is narrow enough that both groups cannot fit on one row
- **THEN** the action-controls group SHALL wrap to a new row
- **AND** the cancel and save buttons SHALL remain together on the same row

### Requirement: Photo tool button

The tool-controls group SHALL contain a photo control that changes between inactive and active states.

#### Scenario: Photo button inactive

- **WHEN** no photo is attached
- **THEN** the toolbar SHALL show a subtle photo-attachment control
- **AND** activating it SHALL open the file picker
- **AND** accepted types SHALL be `image/jpeg, image/png, image/webp`

#### Scenario: Photo button active (compound group)

- **WHEN** a photo is attached
- **THEN** the photo control SHALL become a unified group containing a thumbnail, a remove action, and a replace-photo action
- **AND** the group SHALL read as one coherent control while its actions remain individually identifiable
- **AND** the thumbnail SHALL clearly preview the selected image, including a newly selected local photo
- **AND** the remove action SHALL remove the photo
- **AND** the replace-photo action SHALL open the file picker directly

#### Scenario: Dismissing a photo

- **WHEN** the user clicks the dismiss button on the photo compound group
- **THEN** the photo SHALL be cleared
- **AND** the control SHALL revert to the inactive photo-attachment control
- **AND** any temporary preview resources for the cleared photo SHALL be released

### Requirement: Backdate tool button

The tool-controls group SHALL contain a backdate control that changes between inactive and active states.

#### Scenario: Backdate button inactive

- **WHEN** backdate is not active
- **THEN** the toolbar SHALL show a subtle backdate control
- **AND** activating it SHALL enable backdate mode

#### Scenario: Backdate button active (compound group)

- **WHEN** backdate is active
- **THEN** the backdate control SHALL become a unified group containing a date-and-time input and a remove action
- **AND** the input SHALL default to the current date and time
- **AND** the input SHALL not allow dates or times later than the current moment
- **AND** the group SHALL avoid redundant visual cues

#### Scenario: Dismissing backdate

- **WHEN** the user clicks the dismiss button on the backdate compound group
- **THEN** backdate mode SHALL deactivate
- **AND** the control SHALL revert to the inactive backdate control

### Requirement: Action buttons

The action-controls group SHALL contain cancel and save actions appropriate to adding or editing an event.

#### Scenario: Save button

- **WHEN** the care entry form is opened to add an event
- **THEN** a primary save button SHALL be displayed
- **AND** it SHALL be disabled until an event type is selected
- **AND** clicking it SHALL submit the care event via `POST /api/plants/{id}/care`
- **AND** if a photo is attached, it SHALL be uploaded after event creation
- **AND** on success, the form SHALL close and the saved event SHALL appear in the visible care history

#### Scenario: Save button while editing

- **WHEN** the care entry form is opened to edit an existing care event
- **THEN** clicking the primary save button SHALL update the event via `PUT /api/plants/{id}/care/{event_id}`
- **AND** SHALL apply any requested photo replacement or removal using the care-event photo API
- **AND** on success, the form SHALL close and the updated event SHALL appear in the visible care history

#### Scenario: Cancel button

- **WHEN** the user clicks cancel
- **THEN** unsaved entries and changes SHALL be discarded
- **AND** no pending event or photo mutation SHALL be sent
- **AND** the form SHALL close without changing the care history

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
- **AND** a global toast notification SHALL describe the failure
- **AND** the form SHALL remain open

### Requirement: Care entry edit initialization

When editing a care event, the form SHALL initialize all editable fields from that event while preserving the existing event until the user saves.

#### Scenario: Existing values initialized

- **WHEN** edit mode opens for a care event
- **THEN** its type, notes, and occurrence time are populated in the form
- **AND** its occurrence time is available through the active datetime control

#### Scenario: Existing photo initialized

- **WHEN** the event has a `photo_url`
- **THEN** the form displays the existing photo thumbnail with controls to replace or remove it
- **AND** saving without changing the photo retains the existing photo

#### Scenario: Existing photo removed

- **WHEN** the user marks the existing photo for removal and saves successfully
- **THEN** the event data is updated
- **AND** the existing photo is deleted through the photo API

#### Scenario: Existing photo replaced

- **WHEN** the user selects a replacement photo and saves successfully
- **THEN** the event data is updated
- **AND** the replacement is uploaded through the photo API, replacing the old photo

#### Scenario: Occurrence time cannot be in the future

- **WHEN** the user edits the occurrence time
- **THEN** the datetime control SHALL prevent selecting a future value
- **AND** a future value SHALL fail validation without sending a request

#### Scenario: Edit form disabled offline

- **WHEN** the edit form is open while offline
- **THEN** its save action SHALL be disabled
- **AND** existing field values and staged changes SHALL remain populated
