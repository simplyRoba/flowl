## MODIFIED Requirements

### Requirement: Delete Care Event

The plant detail view SHALL allow deleting individual care events.

#### Scenario: Delete control shown

- **WHEN** the care journal timeline is displayed
- **THEN** each care event shows a delete icon button aligned to the right

#### Scenario: Care event deleted

- **WHEN** the user clicks the delete button on a care event in the timeline
- **THEN** a `DELETE /api/plants/:id/care/:event_id` request is sent
- **AND** the event is removed from the timeline
- **AND** the plant data SHALL be reloaded so that `last_watered`, `watering_status`, and `next_due` reflect the updated care history
