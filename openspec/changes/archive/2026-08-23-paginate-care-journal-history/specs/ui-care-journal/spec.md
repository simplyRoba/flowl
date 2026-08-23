## MODIFIED Requirements

### Requirement: Watering event grouping utility

A shared utility function SHALL group consecutive watering events per plant into collapsible summaries. The function takes care events sorted newest-first together with whether older history remains and returns individual care events or `WateringGroup` items whose identity remains stable as older events are appended.

#### Scenario: Consecutive waterings without notes or photos are grouped

- **WHEN** a plant has 3+ consecutive watering events with no notes and no photos
- **THEN** they SHALL be collapsed into a single `WateringGroup` item containing the count, the earliest loaded date, the latest date, and the loaded original events array

#### Scenario: Watering with notes breaks the streak

- **WHEN** a watering event for a plant has notes
- **THEN** it SHALL render as an individual event and break the grouping streak for that plant

#### Scenario: Watering with photo breaks the streak

- **WHEN** a watering event for a plant has a `photo_url`
- **THEN** it SHALL render as an individual event and break the grouping streak for that plant

#### Scenario: Streak of one is not grouped

- **WHEN** a plant has exactly one consecutive watering event bounded by a breaker or the end of available history
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

#### Scenario: Streak reaches unloaded history

- **GIVEN** older care events remain available
- **WHEN** a plant's oldest loaded streak has no observed breaker before the loaded-history boundary
- **THEN** the streak SHALL be returned as a partial `WateringGroup`, including when only one member is currently loaded
- **AND** its count and earliest date SHALL describe only loaded members

#### Scenario: Partial group receives older members

- **GIVEN** a partial watering group exists
- **WHEN** an older page adds eligible members to that streak
- **THEN** the group SHALL retain the same stable identity anchored to its newest event
- **AND** its count, earliest loaded date, and members SHALL be recomputed

#### Scenario: Partial group becomes complete

- **GIVEN** a partial watering group exists
- **WHEN** loaded history reaches a breaker for that plant or reaches the end of all history
- **THEN** the group SHALL no longer be marked partial
- **AND** a completed one-member streak SHALL render as an individual event

### Requirement: Watering group summary display

Grouped watering events SHALL display as a summary row with an accessible chevron disclosure control in both the global care journal and plant detail timeline, and the summary SHALL distinguish complete groups from groups that may continue into unloaded history.

#### Scenario: Summary row content

- **WHEN** a complete `WateringGroup` is rendered
- **THEN** it SHALL display the plant name on the global page, a watering icon, the exact count, and the complete date range (e.g. "Watered 5 times, Feb 1 - Mar 14")
- **AND** a chevron icon SHALL indicate the group can be expanded

#### Scenario: Partial summary row content

- **WHEN** a partial `WateringGroup` is rendered
- **THEN** it SHALL display its loaded count as an inexact value such as `5+`
- **AND** it SHALL indicate that the streak continues into older entries
- **AND** its displayed date range SHALL cover only the currently loaded members

#### Scenario: Expand group

- **WHEN** the user activates a watering group's chevron disclosure control
- **THEN** the individual loaded watering events within the group SHALL be revealed inline below the summary
- **AND** each expanded event SHALL show its individual date

#### Scenario: Collapse group

- **WHEN** the user activates an expanded watering group's chevron disclosure control
- **THEN** the individual events SHALL be hidden and only the summary row remains

#### Scenario: Expand state is transient

- **WHEN** the user expands a group
- **THEN** the expand/collapse state SHALL be local component state only
- **AND** it SHALL NOT persist in the URL or any store

#### Scenario: Expanded partial group grows

- **GIVEN** a partial group is expanded
- **WHEN** an older page adds members to that group
- **THEN** the group SHALL remain expanded
- **AND** the newly loaded members SHALL appear in the expanded list

### Requirement: Skeleton loading for global care journal

The global care journal SHALL display skeleton shimmer lines while its initial care-event page is being fetched and SHALL preserve loaded content while fetching older pages.

#### Scenario: Loading state shown

- **WHEN** the global care journal is fetching its initial event page
- **THEN** skeleton shimmer lines SHALL be displayed in place of the event list

#### Scenario: Loading state replaced by content

- **WHEN** the initial events have finished loading
- **THEN** the skeleton shimmer lines SHALL be replaced by the actual event list or empty state

#### Scenario: Older page loading state shown

- **GIVEN** the global care journal already displays events
- **WHEN** an older page is being fetched
- **THEN** the loaded event list SHALL remain visible
- **AND** the continuation control SHALL show a loading state

#### Scenario: Shared skeleton styles

- **WHEN** skeleton loading is used
- **THEN** it SHALL use the shared `.shimmer` class from `skeletons.css` rather than component-scoped styles

### Requirement: Global Care Log Page

The route `/care-journal` SHALL display a bounded, progressively loaded feed of care events across all plants and apply watering event grouping to the timeline.

#### Scenario: Events displayed

- **WHEN** the user navigates to `/care-journal`
- **THEN** the page fetches the newest care events from `GET /api/care`
- **AND** displays loaded events grouped by day (e.g. "Today", "Yesterday", "Feb 11, 2026")
- **AND** each event shows the plant name, event type icon, type label, and notes if present
- **AND** consecutive watering events per plant are collapsed into group summaries

#### Scenario: Filter by event type (multi-select)

- **WHEN** the user clicks a type filter chip (Watered, Fertilized, Repotted, Pruned, Custom, AI Consultation)
- **THEN** that type is toggled on or off in the active filter set
- **AND** loaded events and pagination state are reset
- **AND** the event list reloads from the newest page showing only events matching the selected types
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
- **THEN** the filter state returns to unfiltered with no `type` parameter
- **AND** the "All" chip appears active

#### Scenario: Filter state persisted in URL

- **WHEN** type filters are active
- **THEN** the URL SHALL contain `type` query parameters for each selected type (e.g. `?type=watered&type=fertilized`)
- **AND** reloading the page SHALL restore the filter state from the URL
- **AND** the URL is shareable and bookmarkable

#### Scenario: Filter state cleared from URL

- **WHEN** no type filters are active (unfiltered state)
- **THEN** the URL SHALL NOT contain a `type` query parameter

#### Scenario: URL updates without history pollution

- **WHEN** the user toggles a filter chip
- **THEN** the URL SHALL be updated using `replaceState` with no new browser history entry

#### Scenario: Initial event page is bounded

- **WHEN** the global care journal page loads or its filters change
- **THEN** it SHALL request at most 500 raw care events
- **AND** it SHALL retain the response's continuation state rather than requesting all history at once

#### Scenario: All events loaded

- **GIVEN** the user continues loading older entries
- **WHEN** the API reports that no older matching events remain
- **THEN** all matching care events SHALL be present in the timeline
- **AND** they SHALL have been fetched across one or more bounded requests

#### Scenario: No events

- **WHEN** the initial response contains no care events for the selected filters
- **THEN** the page displays an empty state message

#### Scenario: Navigate to plant

- **WHEN** the user clicks a plant name in the global log
- **THEN** the app navigates to that plant's detail view

## ADDED Requirements

### Requirement: Hybrid Care Journal History Loading

The global care journal SHALL provide bounded access to all older matching events through a continuation control, using automatic loading only when the rendered journal already overflows its scroll container.

#### Scenario: More history without scrolling

- **GIVEN** the API reports that older matching events exist
- **AND** the rendered journal does not overflow its scroll container
- **THEN** a "Load older entries" control SHALL be visible and enabled
- **AND** merely being visible SHALL NOT automatically fetch another page

#### Scenario: User loads older entries manually

- **GIVEN** older matching events exist
- **WHEN** the user activates "Load older entries"
- **THEN** the next page of at most 500 raw events SHALL be requested using the current cursor and filters
- **AND** the returned events SHALL be appended once in chronological order
- **AND** grouping SHALL be recomputed across all loaded events

#### Scenario: Scrollable journal loads automatically

- **GIVEN** older matching events exist
- **AND** the rendered journal overflows its scroll container
- **WHEN** the continuation control approaches the visible bottom of that container
- **THEN** the next page SHALL load automatically
- **AND** the continuation control SHALL remain usable as a manual fallback

#### Scenario: Compaction keeps the continuation control visible

- **GIVEN** an automatically loaded page compacts without moving the continuation control out of view
- **AND** older matching events still exist
- **THEN** another page SHALL NOT load automatically while the control remains continuously visible
- **AND** automatic loading SHALL re-arm only after the control leaves and re-enters view
- **AND** the user MAY still load the next page manually

#### Scenario: Scrollability changes

- **GIVEN** older matching events exist
- **WHEN** viewport resizing or group expansion changes whether the journal overflows
- **THEN** automatic loading eligibility SHALL be recalculated
- **AND** a non-overflowing journal SHALL return to manual-only loading

#### Scenario: Concurrent continuation trigger

- **GIVEN** an older-page request is already in progress
- **WHEN** the observer or user triggers continuation again
- **THEN** no overlapping request SHALL be started
- **AND** no event SHALL be appended more than once

#### Scenario: Older-page request fails

- **GIVEN** the journal already displays events
- **WHEN** loading an older page fails
- **THEN** the loaded events SHALL remain visible and unchanged
- **AND** automatic retry SHALL pause to prevent a request loop
- **AND** the continuation control SHALL expose the error and allow a user-initiated retry from the same cursor

#### Scenario: Cursor event was deleted

- **GIVEN** the journal already displays events
- **WHEN** an older-page request reports that its cursor event no longer exists
- **THEN** the loaded events SHALL remain visible
- **AND** the continuation control SHALL offer a journal refresh instead of retrying the invalid cursor
- **AND** activating refresh SHALL restart loading from the newest page with the current filters

#### Scenario: Filters change during a request

- **GIVEN** a care-event request is in progress
- **WHEN** the selected event-type filters change
- **THEN** any response belonging to the previous filter state SHALL NOT alter the new filtered timeline
- **AND** loading SHALL restart from the newest page for the new filters

#### Scenario: End of history reached

- **WHEN** the API reports that no older matching events remain
- **THEN** automatic continuation SHALL stop
- **AND** the "Load older entries" control SHALL no longer be displayed
