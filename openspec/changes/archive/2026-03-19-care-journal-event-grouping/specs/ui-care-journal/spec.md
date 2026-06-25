## ADDED Requirements

### Requirement: Watering event grouping utility

A shared utility function SHALL group consecutive watering events per plant into collapsible summaries. The function takes a `CareEvent[]` (sorted newest-first) and returns `Array<CareEvent | WateringGroup>`.

#### Scenario: Consecutive waterings without notes or photos are grouped

- **WHEN** a plant has 3+ consecutive watering events with no notes and no photos
- **THEN** they SHALL be collapsed into a single `WateringGroup` item containing the count, the earliest date, the latest date, and the original events array

#### Scenario: Watering with notes breaks the streak

- **WHEN** a watering event for a plant has notes
- **THEN** it SHALL render as an individual event and break the grouping streak for that plant

#### Scenario: Watering with photo breaks the streak

- **WHEN** a watering event for a plant has a photo_url
- **THEN** it SHALL render as an individual event and break the grouping streak for that plant

#### Scenario: Streak of one is not grouped

- **WHEN** a plant has only one consecutive watering event (surrounded by other event types or at the boundary)
- **THEN** it SHALL render as an individual event, not as a group

#### Scenario: Streak of two is grouped

- **WHEN** a plant has exactly two consecutive watering events without notes or photos
- **THEN** they SHALL be collapsed into a `WateringGroup`

#### Scenario: Non-watering events do not break other plants

- **WHEN** the timeline contains events from multiple plants interleaved
- **THEN** each plant's watering streak SHALL be tracked independently
- **AND** events from other plants between two waterings of plant A SHALL NOT break plant A's streak

#### Scenario: Non-watering event for same plant breaks streak

- **WHEN** a non-watering event for plant A appears between two watering events for plant A
- **THEN** plant A's watering streak SHALL be broken at that point

### Requirement: Watering group summary display

Grouped watering events SHALL display as a collapsible summary row in both the global care journal and plant detail timeline.

#### Scenario: Summary row content

- **WHEN** a `WateringGroup` is rendered
- **THEN** it SHALL display the plant name (on global page), a watering icon, the count, and the date range (e.g. "Watered 5 times, Feb 1 - Mar 14")
- **AND** a chevron icon SHALL indicate the group can be expanded

#### Scenario: Expand group

- **WHEN** the user clicks/taps a group summary row
- **THEN** the individual watering events within the group SHALL be revealed inline below the summary
- **AND** each expanded event SHALL show its individual date

#### Scenario: Collapse group

- **WHEN** the user clicks/taps an expanded group summary row
- **THEN** the individual events SHALL be hidden and only the summary row remains

#### Scenario: Expand state is transient

- **WHEN** the user expands a group
- **THEN** the expand/collapse state SHALL be local component state only
- **AND** it SHALL NOT persist in the URL or any store

### Requirement: Skeleton loading for global care journal

The global care journal SHALL display skeleton shimmer lines while care events are being fetched, using shared skeleton styles.

#### Scenario: Loading state shown

- **WHEN** the global care journal page is loading events
- **THEN** skeleton shimmer lines SHALL be displayed in place of the event list

#### Scenario: Loading state replaced by content

- **WHEN** the events have finished loading
- **THEN** the skeleton shimmer lines SHALL be replaced by the actual event list (or empty state)

#### Scenario: Shared skeleton styles

- **WHEN** skeleton loading is used
- **THEN** it SHALL use the shared `.shimmer` class from `skeletons.css` rather than component-scoped styles

### Requirement: Global care journal grouping integration

The global care journal page SHALL apply watering event grouping to its event list.

#### Scenario: Group summary in global timeline

- **WHEN** the global care journal is rendered
- **THEN** the event list SHALL be processed through the grouping utility before display
- **AND** group summaries SHALL appear inline within the day-grouped timeline

#### Scenario: Plant name shown in global group summary

- **WHEN** a group summary is rendered on the global care journal
- **THEN** the plant name SHALL be displayed as a link to the plant detail page

### Requirement: Plant detail timeline grouping integration

The plant detail care journal section SHALL apply the same watering event grouping.

#### Scenario: Group summary in plant timeline

- **WHEN** the plant detail timeline is rendered
- **THEN** the event list SHALL be processed through the grouping utility before display

#### Scenario: Plant name omitted in plant detail group summary

- **WHEN** a group summary is rendered on the plant detail page
- **THEN** the plant name SHALL be omitted (since the context is already a single plant)

## MODIFIED Requirements

### Requirement: Global Care Log Page

The route `/care-journal` SHALL display a feed of care events across all plants.

#### Scenario: Events displayed

- **WHEN** the user navigates to `/care-journal`
- **THEN** the page fetches all care events from `GET /api/care`
- **AND** displays events grouped by day (e.g., "Today", "Yesterday", "Feb 11, 2026")
- **AND** each event shows the plant name, event type icon, type label, and notes (if present)
- **AND** consecutive watering events per plant are collapsed into group summaries

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

#### Scenario: All events loaded

- **WHEN** the global care journal page loads
- **THEN** all care events SHALL be fetched in a single request
- **AND** no infinite scroll or cursor-based pagination SHALL be used

#### Scenario: No events

- **WHEN** no care events exist across any plant (or for the selected filters)
- **THEN** the page displays an empty state message

#### Scenario: Navigate to plant

- **WHEN** the user clicks a plant name in the global log
- **THEN** the app navigates to that plant's detail view

## REMOVED Requirements

### Requirement: Infinite scroll

**Reason**: Replaced by load-all approach to enable accurate client-side grouping of watering events. With infinite scroll, group counts change as pages load, creating a confusing experience.

**Migration**: The global care journal fetches all events in a single request. The sentinel observer and cursor-based pagination are removed.
