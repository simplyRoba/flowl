## ADDED Requirements

### Requirement: Care entry form on detail view

The plant detail view SHALL render the care entry form using the `CareEntryForm` component.

#### Scenario: Add log entry button

- **WHEN** the plant detail view is rendered
- **AND** the care entry form is not visible
- **THEN** an "Add log entry" button SHALL be displayed below the care journal
- **AND** clicking it SHALL show the `CareEntryForm` component

#### Scenario: Care entry form rendered via component

- **WHEN** the user clicks the "Add log entry" button
- **THEN** the plant detail page SHALL render `<CareEntryForm plantId={plant.id} />` inline
- **AND** the page SHALL NOT contain any form state variables (event type, notes, photo, backdate, submitting) — all state SHALL be encapsulated within the component

#### Scenario: Form submit reloads events

- **WHEN** the `CareEntryForm` emits `onsubmit`
- **THEN** the plant detail page SHALL reload care events
- **AND** the form SHALL be hidden

#### Scenario: Form cancel hides form

- **WHEN** the `CareEntryForm` emits `oncancel`
- **THEN** the form SHALL be hidden
