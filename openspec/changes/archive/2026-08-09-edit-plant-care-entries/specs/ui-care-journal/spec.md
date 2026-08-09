## ADDED Requirements

### Requirement: Edit Care Event on Plant Detail Timeline

The plant detail care journal SHALL allow an online user to edit each individual care event. The global care journal SHALL remain read-only and SHALL NOT display or initiate care-event editing.

#### Scenario: Edit control shown on plant detail

- **WHEN** an individual care event is displayed in the plant detail timeline while online
- **THEN** the event shows an edit control alongside its existing actions

#### Scenario: Grouped watering entry edited

- **GIVEN** watering events are displayed as a collapsed group on the plant detail page
- **WHEN** the user expands the group
- **THEN** each revealed event shows its own edit control
- **AND** the user can edit that specific event

#### Scenario: Edit submitted

- **WHEN** the user saves valid changes to a care event on the plant detail page
- **THEN** a `PUT /api/plants/:id/care/:event_id` request is sent
- **AND** the plant and care-event timeline are reloaded
- **AND** the timeline reflects the event's updated type, notes, occurrence time, and photo

#### Scenario: Edit cancelled

- **WHEN** the user cancels editing a care event
- **THEN** the edit form is hidden
- **AND** no update or photo mutation request is sent

#### Scenario: Edit fails

- **WHEN** a care-event edit request fails
- **THEN** the entered changes remain available in the form
- **AND** a global toast notification describes the failure

#### Scenario: Global journal remains read-only

- **WHEN** the user views `/care-journal`
- **THEN** no care-event edit control or edit form is displayed
- **AND** the page sends no care-event update request

### Requirement: Update Care Event API Client

The frontend API client SHALL provide a typed function for updating a care event's editable data.

#### Scenario: Update care event

- **WHEN** `updateCareEvent(plantId, eventId, data)` is called
- **THEN** a `PUT` request is made to `/api/plants/{plantId}/care/{eventId}` with the editable event data
- **AND** the updated `CareEvent` is returned
