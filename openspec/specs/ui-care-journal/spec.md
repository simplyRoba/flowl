## Purpose

Care journal UI — timeline on plant detail view, inline log form, delete actions, global care log page with filtering and infinite scroll, API client and store.

## Requirements

### Requirement: Care Journal Timeline

The plant detail view SHALL display a care journal section showing a chronological timeline of all fetched care events.

#### Scenario: Care events displayed

- **WHEN** the plant detail view is rendered
- **AND** the plant has care events
- **THEN** a "Care Journal" section is shown below the watering card
- **AND** all fetched care events are displayed immediately
- **AND** care events are listed newest first
- **AND** each event shows an icon for the event type, the type label, the date, and notes (if present)

#### Scenario: No care events

- **WHEN** the plant detail view is rendered
- **AND** the plant has no care events
- **THEN** the care journal section shows an empty state message

#### Scenario: Event type icons

- **WHEN** a care event is displayed
- **THEN** the icon corresponds to the event type: droplet for `watered`, leaf for `fertilized`, shovel for `repotted`, scissors for `pruned`, pencil for `custom`, sparkles for `ai-consultation`

#### Scenario: Event limit

- **WHEN** the plant has more than 20 care events
- **THEN** all fetched care events are shown immediately
- **AND** no "Show more" control is displayed

### Requirement: Watering event grouping utility

A shared utility function SHALL group consecutive watering events per plant into collapsible summaries. The function takes care events sorted newest-first together with whether older history remains and returns individual care events or `WateringGroup` items whose identity remains stable as older events are appended.

#### Scenario: Consecutive waterings without notes or photos are grouped

- **WHEN** a plant has 3+ consecutive watering events with no notes and no photos
- **THEN** they SHALL be collapsed into a single `WateringGroup` item containing the count, the earliest loaded date, the latest date, and the loaded original events array

#### Scenario: Watering with notes breaks the streak

- **WHEN** a watering event for a plant has notes
- **THEN** it SHALL render as an individual event and break the grouping streak for that plant

#### Scenario: Watering with photo breaks the streak

- **WHEN** a watering event for a plant has a photo_url
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

#### Scenario: Watered type available

- **WHEN** the log care form is displayed
- **THEN** the `watered` event type SHALL be available as a chip alongside fertilized, repotted, pruned, and custom

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

### Requirement: AI consultation event styling

The `ai-consultation` event type SHALL have distinct visual treatment in both the plant detail timeline and global care journal.

#### Scenario: AI consultation icon

- **WHEN** an `ai-consultation` care event is displayed in any timeline
- **THEN** the event icon SHALL be `Sparkles` (from lucide-svelte)

#### Scenario: AI consultation color

- **WHEN** an `ai-consultation` care event is displayed in the global care journal
- **THEN** the icon background SHALL use `var(--color-ai)` as its accent color

#### Scenario: AI consultation label

- **WHEN** an `ai-consultation` care event is displayed
- **THEN** the event type label SHALL be "AI Consultation"

### Requirement: Care event photo in plant detail timeline

Care events with a `photo_url` SHALL display a clickable thumbnail in the plant detail timeline, using a thumbnail variant sized for the display context.

#### Scenario: Event with photo

- **WHEN** a care event in the plant detail timeline has a `photo_url`
- **THEN** a 72px rounded thumbnail (`object-fit: cover`) SHALL be displayed using the 200px thumbnail (derived via `thumbUrl(photo_url, 200)`)
- **AND** the thumbnail SHALL float to the right of the text content, with text wrapping beside it on wider viewports
- **AND** clicking the thumbnail SHALL open the PhotoLightbox with the original `photo_url` at full resolution

#### Scenario: Event without photo

- **WHEN** a care event has no `photo_url`
- **THEN** no thumbnail space SHALL be rendered

### Requirement: Photo upload in log care form

The inline log care form SHALL allow attaching an optional photo to the care event.

#### Scenario: Upload control displayed

- **WHEN** the log care form is visible
- **THEN** a compact photo upload control (camera icon label with hidden file input) SHALL appear below the notes textarea
- **AND** the control SHALL accept `image/jpeg`, `image/png`, `image/webp`

#### Scenario: Photo preview shown

- **WHEN** the user selects a photo via the upload control
- **THEN** a ~64px thumbnail preview with a remove button SHALL replace the upload control
- **AND** clicking the remove button SHALL clear the staged photo and restore the upload control

#### Scenario: Submit with photo

- **WHEN** the user submits the log form with a photo staged
- **THEN** the care event SHALL be created first via `POST /api/plants/:id/care`
- **AND** then the photo SHALL be uploaded via `POST /api/plants/:id/care/:event_id/photo`
- **AND** the timeline SHALL refresh to show the new event with its photo

#### Scenario: Cancel clears photo

- **WHEN** the user cancels the log form with a photo staged
- **THEN** the staged photo SHALL be cleared

### Requirement: Care event photo in global care journal

Care events with a `photo_url` SHALL display a clickable thumbnail in the global care journal page, using a thumbnail variant sized for the display context.

#### Scenario: Event with photo

- **WHEN** a care event in the global journal has a `photo_url`
- **THEN** a 80px rounded thumbnail (`object-fit: cover`) SHALL be displayed using the 200px thumbnail (derived via `thumbUrl(photo_url, 200)`)
- **AND** the thumbnail SHALL float to the right of the text content, with text wrapping beside it on wider viewports
- **AND** clicking the thumbnail SHALL open a PhotoLightbox with the original `photo_url` at full resolution

#### Scenario: Event without photo

- **WHEN** a care event in the global journal has no `photo_url`
- **THEN** no thumbnail space SHALL be rendered

### Requirement: Care event photo API client functions

The frontend API client SHALL include `photo_url` on the `CareEvent` type and provide functions for care event photo upload and delete.

#### Scenario: CareEvent includes photo_url

- **WHEN** the `CareEvent` TypeScript interface is defined
- **THEN** it SHALL include `photo_url: string | null`

#### Scenario: Upload care event photo

- **WHEN** `uploadCareEventPhoto(plantId, eventId, file)` is called
- **THEN** a `POST` multipart request SHALL be made to `/api/plants/{plantId}/care/{eventId}/photo` with the file in a FormData `"file"` field
- **AND** the updated `CareEvent` SHALL be returned

#### Scenario: Delete care event photo

- **WHEN** `deleteCareEventPhoto(plantId, eventId)` is called
- **THEN** a `DELETE` request SHALL be made to `/api/plants/{plantId}/care/{eventId}/photo`

### Requirement: Care journal offline message

The global care journal page SHALL display an offline-specific message instead of a generic error when the network is unavailable.

#### Scenario: Offline message shown when fetch fails offline

- **WHEN** the care journal page attempts to load events
- **AND** the fetch fails
- **AND** `navigator.onLine` is `false`
- **THEN** the page SHALL display a translated offline message instead of the generic load error text

#### Scenario: Generic error shown when fetch fails online

- **WHEN** the care journal page attempts to load events
- **AND** the fetch fails
- **AND** `navigator.onLine` is `true`
- **THEN** the page SHALL display the existing generic error text from `resolveError()`

#### Scenario: Skeleton shown before offline determination

- **WHEN** the care journal page is loading events
- **THEN** the existing skeleton shimmer lines SHALL be displayed while the fetch is in progress
- **AND** the offline message SHALL only appear after the fetch fails

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
