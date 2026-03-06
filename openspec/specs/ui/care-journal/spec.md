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
- **THEN** the icon corresponds to the event type: droplet for `watered`, leaf for `fertilized`, shovel for `repotted`, scissors for `pruned`, pencil for `custom`, sparkles for `ai-consultation`

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

#### Scenario: Delete control shown

- **WHEN** the care journal timeline is displayed
- **THEN** each care event shows a delete icon button aligned to the right

#### Scenario: Care event deleted

- **WHEN** the user clicks the delete button on a care event in the timeline
- **THEN** a `DELETE /api/plants/:id/care/:event_id` request is sent
- **AND** the event is removed from the timeline
- **AND** the plant data SHALL be reloaded so that `last_watered`, `watering_status`, and `next_due` reflect the updated care history

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
- **AND** the thumbnail SHALL float to the right of the text content, with the time remaining pinned to the top-right of the entry
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
