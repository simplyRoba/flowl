## ADDED Requirements

### Requirement: Plant Detail Care Event Edit Lifecycle

The plant detail page SHALL own which care event is being edited while the `CareEntryForm` component SHALL own all editable form fields and submission state.

#### Scenario: Open edit form

- **WHEN** the user activates an individual care event's edit control
- **THEN** the page renders `CareEntryForm` in edit mode for that event
- **AND** any add-entry form or other care-event edit form is hidden

#### Scenario: Successful edit refreshes dependent data

- **WHEN** the edit form reports a successful submission
- **THEN** the page hides the form
- **AND** reloads both the plant and its care events
- **AND** `last_watered`, `watering_status`, and `next_due` reflect the updated history

#### Scenario: Cancel edit

- **WHEN** the edit form reports cancellation
- **THEN** the page hides the form without changing the displayed event

#### Scenario: Edit action disabled offline

- **WHEN** the plant detail page is offline
- **THEN** every care-event edit control is visually disabled
- **AND** activating it SHALL NOT open the edit form or send a mutation request

#### Scenario: Connectivity is lost while editing

- **WHEN** the device becomes offline while an edit form is open
- **THEN** the form's save action is disabled
- **AND** the user's populated form state is retained
