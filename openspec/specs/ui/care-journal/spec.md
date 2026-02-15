## Purpose

Care journal UI — timeline on plant detail view, inline log form, delete actions, global care log page with filtering and infinite scroll, API client and store.

## Requirements

### Requirement: Care Journal Timeline

The plant detail view SHALL display a care journal section showing a chronological timeline of care events.

#### Scenario: Care events displayed

- **WHEN** the plant detail view is rendered
- **AND** the plant has care events
- **THEN** a "Care Journal" section is shown below the watering card
- **AND** care events are grouped by day (e.g., "Today", "Yesterday", "Feb 10") and listed newest first within each group
- **AND** each event shows an icon for the event type, the type label, the date, and notes (if present)

#### Scenario: No care events

- **WHEN** the plant detail view is rendered
- **AND** the plant has no care events
- **THEN** the care journal section shows an empty state message

#### Scenario: Event type icons

- **WHEN** a care event is displayed
- **THEN** the icon corresponds to the event type: droplet for `watered`, leaf for `fertilized`, shovel for `repotted`, scissors for `pruned`, pencil for `custom`

#### Scenario: Event limit

- **WHEN** the plant has more than 20 care events
- **THEN** only the 20 most recent events are shown initially
- **AND** a "Show more" link is displayed to load the rest

### Requirement: Log Care Action

The plant detail view SHALL provide an "+ Add log entry" action for manually recording non-watering care events.

#### Scenario: Log care form displayed

- **WHEN** the user clicks the "+ Add log entry" link below the care journal timeline
- **THEN** an inline form appears with event type options (Fertilized, Repotted, Pruned, Custom) and an optional notes field

#### Scenario: Care event submitted

- **WHEN** the user selects an event type, optionally enters notes, and clicks Save
- **THEN** a `POST /api/plants/:id/care` request is sent with the selected type and notes
- **AND** the care journal timeline refreshes to include the new event

#### Scenario: Form cancelled

- **WHEN** the user clicks Cancel on the log care form
- **THEN** the form is hidden without sending a request

#### Scenario: Watered type excluded

- **WHEN** the log care form is displayed
- **THEN** the `watered` event type is NOT available as an option
- **AND** users are expected to use the "Water now" button for watering

### Requirement: Delete Care Event

The plant detail view SHALL allow deleting individual care events.

#### Scenario: Care event deleted

- **WHEN** the user clicks the delete button on a care event in the timeline
- **THEN** a `DELETE /api/plants/:id/care/:event_id` request is sent
- **AND** the event is removed from the timeline

### Requirement: Global Care Log Page

The route `/log` SHALL display a paginated feed of care events across all plants.

#### Scenario: Events displayed

- **WHEN** the user navigates to `/log`
- **THEN** the page fetches care events from `GET /api/care`
- **AND** displays events grouped by day (e.g., "Today", "Yesterday", "Feb 11, 2026")
- **AND** each event shows the plant name, event type icon, type label, time, and notes (if present)

#### Scenario: Filter by event type

- **WHEN** the user clicks a filter chip (All, Watered, Fertilized, Repotted, Pruned, Custom)
- **THEN** the event list reloads showing only events of the selected type
- **AND** the "All" chip shows all event types

#### Scenario: Infinite scroll

- **WHEN** the user scrolls near the bottom of the event list
- **AND** more events are available
- **THEN** the next page is fetched automatically using the cursor (`before` parameter)
- **AND** new events are appended to the list

#### Scenario: No events

- **WHEN** no care events exist across any plant (or for the selected filter)
- **THEN** the page displays an empty state message

#### Scenario: Navigate to plant

- **WHEN** the user clicks a plant name in the global log
- **THEN** the app navigates to that plant's detail view

### Requirement: Care Events API Client

The frontend API client SHALL provide typed functions for care event operations.

#### Scenario: Fetch care events for plant

- **WHEN** `fetchCareEvents(plantId)` is called
- **THEN** a `GET` request is made to `/api/plants/{plantId}/care`
- **AND** a `CareEvent[]` array is returned

#### Scenario: Fetch global care events

- **WHEN** `fetchAllCareEvents(limit?, before?, type?)` is called
- **THEN** a `GET` request is made to `/api/care` with optional query parameters (`limit`, `before`, `type`)
- **AND** a `{ events: CareEvent[], has_more: boolean }` object is returned

#### Scenario: Create care event

- **WHEN** `createCareEvent(plantId, data)` is called
- **THEN** a `POST` request is made to `/api/plants/{plantId}/care`
- **AND** the created `CareEvent` is returned

#### Scenario: Delete care event

- **WHEN** `deleteCareEvent(plantId, eventId)` is called
- **THEN** a `DELETE` request is made to `/api/plants/{plantId}/care/{eventId}`

### Requirement: Care Events Store

The frontend SHALL provide a care events store that manages care event state for the current plant.

#### Scenario: Load care events

- **WHEN** `loadCareEvents(plantId)` is called
- **THEN** the store is populated with the plant's care events

#### Scenario: Add care event

- **WHEN** `addCareEvent(plantId, data)` is called
- **THEN** the API is called and the new event is added to the store

#### Scenario: Remove care event

- **WHEN** `removeCareEvent(plantId, eventId)` is called
- **THEN** the API is called and the event is removed from the store
