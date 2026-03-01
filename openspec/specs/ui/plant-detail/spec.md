## Purpose

Plant detail view — full plant information display with photo hero, watering section, care info card, lightbox, Ask AI integration, and plant API client/store functions.

## Requirements

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, and notes
- **AND** the detail grid contains a "Watering" card and a "Care Info" card

#### Scenario: Care Info card content

- **WHEN** the plant detail view is rendered
- **THEN** the "Care Info" card SHALL always display the light needs row with icon and label
- **AND** for each non-null care info field (`difficulty`, `pet_safety`, `growth_speed`, `soil_type`), a labeled row SHALL be displayed in the "Care Info" card
- **AND** null care info fields SHALL be omitted (no empty or "N/A" rows)

#### Scenario: Soil moisture on Watering card

- **WHEN** the plant detail view is rendered
- **AND** the plant has a non-null `soil_moisture` value
- **THEN** a "Soil moisture" row SHALL be displayed in the "Watering" card below the "Next due" row
- **AND** the row SHALL show a human-readable label ("Prefers dry", "Moderate", "Keeps moist")

#### Scenario: Soil moisture not set

- **WHEN** the plant has `soil_moisture` = null
- **THEN** no "Soil moisture" row SHALL appear in the "Watering" card

#### Scenario: Care Info card with no optional attributes set

- **WHEN** a plant has no `difficulty`, `pet_safety`, `growth_speed`, or `soil_type` set
- **THEN** the "Care Info" card SHALL display only the light needs row

#### Scenario: Care journal delete control

- **WHEN** the plant detail view is rendered
- **THEN** each care journal entry shows a delete control

#### Scenario: Edit action

- **WHEN** the user clicks the edit button on the detail view
- **THEN** the app navigates to `/plants/1/edit`

#### Scenario: Delete action

- **WHEN** the user clicks the delete button on the detail view
- **THEN** a `ModalDialog` is shown in confirm mode with danger variant
- **AND** the dialog message includes the plant name
- **AND** deletion only proceeds when the user confirms

#### Scenario: Plant not found

- **WHEN** the user navigates to `/plants/999`
- **AND** the API returns 404
- **THEN** the page displays a "Plant not found" message

#### Scenario: Widescreen detail layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the detail page max-width SHALL be 960px (increased from 800px)
- **AND** the hero photo/icon SHALL be 100px (increased from 80px)

#### Scenario: Ask AI button displayed

- **WHEN** the plant detail view is rendered
- **AND** `GET /api/ai/status` returns `{ "enabled": true }`
- **THEN** an "Ask AI" button with a sparkle icon SHALL be displayed in the hero section next to the "Water now" button
- **AND** the button SHALL use the AI accent color (`--color-ai`)

#### Scenario: Ask AI button hidden when AI disabled

- **WHEN** `GET /api/ai/status` returns `{ "enabled": false }` or fails
- **THEN** the "Ask AI" button SHALL NOT be rendered

#### Scenario: Ask AI opens chat drawer

- **WHEN** the user clicks the "Ask AI" button
- **THEN** the `ChatDrawer` component SHALL open with the current plant's data

#### Scenario: Mobile action bar hidden during chat

- **WHEN** the chat drawer is open on mobile (viewport <= 768px)
- **THEN** the page action bar (Back / Edit / Delete) SHALL be hidden
- **AND** the bottom sheet SHALL sit directly above the bottom nav bar

#### Scenario: Mobile action bar restored on chat close

- **WHEN** the chat drawer is closed on mobile
- **THEN** the page action bar SHALL reappear

### Requirement: Photo Display on Detail View

The plant detail hero section SHALL display the plant's photo when available.

#### Scenario: Plant has photo

- **WHEN** a plant has a `photo_url`
- **THEN** the detail hero shows the photo instead of the emoji icon

#### Scenario: Plant has no photo

- **WHEN** a plant has no `photo_url`
- **THEN** the detail hero shows the Noto emoji icon as before

### Requirement: Detail image lightbox

The plant detail view SHALL allow opening a lightbox overlay for the hero photo when a photo is available.

#### Scenario: Open lightbox from detail photo

- **WHEN** the user clicks or taps the hero photo on `/plants/[id]`
- **THEN** a lightbox overlay opens displaying the same photo at an enlarged size
- **AND** the background content is visually dimmed
- **AND** page scrolling is disabled while the lightbox is open

#### Scenario: Close lightbox

- **WHEN** the user presses the Escape key
- **OR** clicks the overlay backdrop outside the image
- **THEN** the lightbox closes
- **AND** page scrolling is restored

#### Scenario: No photo available

- **WHEN** the plant has no `photo_url`
- **THEN** the hero displays the emoji icon as before
- **AND** no lightbox opens when the hero is clicked

### Requirement: Lightbox zoom and pan

The lightbox SHALL support zooming and panning of the photo for detailed inspection.

#### Scenario: Zoom in and out

- **WHEN** the user uses the mouse wheel or a pinch gesture on the lightbox image
- **THEN** the image scale increases or decreases within defined limits

#### Scenario: Pan a zoomed image

- **WHEN** the image is zoomed in and the user drags the image
- **THEN** the image pans within its bounds without exposing empty background beyond the overlay

### Requirement: Plant Detail Watering Section

The plant detail view SHALL display watering status and a "Water now" action.

#### Scenario: Watering status displayed

- **WHEN** the plant detail view is rendered
- **THEN** the watering info card shows the current watering status (`ok`, `due`, or `overdue`) with a colored indicator
- **AND** the last watered date is shown (or "Never" if null)
- **AND** the next due date is shown (or "N/A" if never watered)
- **AND** the watering interval is shown

#### Scenario: Water now action

- **WHEN** the user clicks the "Water now" button on the plant detail view
- **THEN** a `POST /api/plants/:id/water` request is sent
- **AND** the view refreshes to show updated watering status
- **AND** the `last_watered` field updates to the current time

#### Scenario: Water now when already ok

- **WHEN** the user clicks "Water now" on a plant that is already `ok`
- **THEN** the watering is recorded (last_watered updates)
- **AND** the status remains `ok` with a new next_due date

### Requirement: API Client

A shared API client module SHALL provide typed functions for all plant and location API calls, handling JSON serialization and error responses.

#### Scenario: API error

- **WHEN** an API call returns an error status
- **THEN** the client extracts the error message and surfaces it to the calling component

### Requirement: API Client — Water Plant

The frontend API client SHALL provide a `waterPlant` function.

#### Scenario: Water plant call

- **WHEN** `waterPlant(1)` is called
- **THEN** a `POST` request is made to `/api/plants/1/water`
- **AND** the updated `Plant` object is returned

### Requirement: Plant Store — Water Plant

The plant store SHALL provide a `waterPlant` function that calls the API and updates the store.

#### Scenario: Store updated after watering

- **WHEN** `waterPlant(1)` is called on the store
- **THEN** the plant list and current plant are updated with the new watering data
