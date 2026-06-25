## MODIFIED Requirements

### Requirement: Global Care Log Page

The route `/care-journal` SHALL display a paginated feed of care events across all plants.

#### Scenario: Events displayed

- **WHEN** the user navigates to `/care-journal`
- **THEN** the page fetches care events from `GET /api/care`
- **AND** displays events grouped by day (e.g., "Today", "Yesterday", "Feb 11, 2026")
- **AND** each event shows the plant name, event type icon, type label, and notes (if present)

#### Scenario: Filter by event type (multi-select)

- **WHEN** the user clicks a type filter chip (Watered, Fertilized, Repotted, Pruned, Custom, AI Consultation)
- **THEN** that type is toggled on or off in the active filter set
- **AND** the event list reloads showing only events matching the selected types
- **AND** multiple chips MAY be active simultaneously

#### Scenario: All chip clears filters

- **WHEN** the user clicks the "All" chip
- **AND** one or more type filters are active
- **THEN** all type filters are cleared
- **AND** the event list reloads showing all event types

#### Scenario: All chip selects all types

- **WHEN** the user clicks the "All" chip
- **AND** no type filters are active (unfiltered state)
- **THEN** all 6 event types SHALL be selected explicitly
- **AND** the user can then toggle individual types off to achieve an "all but X" selection

#### Scenario: All chip appearance

- **WHEN** no type filters are active (unfiltered state)
- **THEN** the "All" chip SHALL appear active
- **WHEN** all 6 types are explicitly selected
- **THEN** the "All" chip SHALL also appear active

#### Scenario: Last type toggled off

- **WHEN** the user toggles off the last remaining active type filter
- **THEN** the filter state returns to unfiltered (no `type` param)
- **AND** the "All" chip appears active

#### Scenario: Filter state persisted in URL

- **WHEN** type filters are active
- **THEN** the URL SHALL contain `type` query parameters for each selected type (e.g., `?type=watered&type=fertilized`)
- **AND** reloading the page SHALL restore the filter state from the URL
- **AND** the URL is shareable/bookmarkable

#### Scenario: Filter state cleared from URL

- **WHEN** no type filters are active (unfiltered state)
- **THEN** the URL SHALL NOT contain a `type` query parameter

#### Scenario: URL updates without history pollution

- **WHEN** the user toggles a filter chip
- **THEN** the URL SHALL be updated using `replaceState` (no new browser history entry)

#### Scenario: Infinite scroll

- **WHEN** the user scrolls near the bottom of the event list
- **AND** more events are available
- **THEN** the next page is fetched automatically using the cursor (`before` parameter)
- **AND** new events are appended to the list

#### Scenario: No events

- **WHEN** no care events exist across any plant (or for the selected filters)
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

- **WHEN** `fetchAllCareEvents(limit?, before?, types?)` is called
- **THEN** a `GET` request is made to `/api/care` with optional query parameters (`limit`, `before`, and a `type` param per entry in `types`)
- **AND** a `{ events: CareEvent[], has_more: boolean }` object is returned

#### Scenario: Create care event

- **WHEN** `createCareEvent(plantId, data)` is called
- **THEN** a `POST` request is made to `/api/plants/{plantId}/care`
- **AND** the created `CareEvent` is returned

#### Scenario: Delete care event

- **WHEN** `deleteCareEvent(plantId, eventId)` is called
- **THEN** a `DELETE` request is made to `/api/plants/{plantId}/care/{eventId}`
