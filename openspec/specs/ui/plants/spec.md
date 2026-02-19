## Purpose

Plant UI screens — dashboard with card grid, photo display, and watering status indicators; plant detail view with photo hero and watering section; add/edit forms with photo upload and interactive controls.

## Requirements

### Requirement: Plants Dashboard

The root route (`/`) SHALL display a grid of plant cards showing each plant's icon, name, and location.

#### Scenario: Plants loaded

- **WHEN** the user navigates to `/`
- **THEN** the page fetches plants from `GET /api/plants`
- **AND** displays a card grid with each plant's Noto emoji icon, name, and location name

#### Scenario: No plants

- **WHEN** the user navigates to `/`
- **AND** no plants exist
- **THEN** the page displays an empty state with an "Add Plant" prompt

#### Scenario: Add plant button

- **WHEN** the user clicks "Add Plant"
- **THEN** the app navigates to `/plants/new`

#### Scenario: Widescreen dashboard layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the dashboard max-width SHALL be 1400px (increased from 1200px)
- **AND** the plant cards SHALL use a full-bleed image layout (240px tall photo area)
- **AND** the card name and location SHALL float over the image via a bottom gradient overlay
- **AND** the grid gap SHALL be 20px

#### Scenario: Dynamic greeting subtitle when plants need attention

- **WHEN** one or more plants have `watering_status` of `due` or `overdue`
- **THEN** the greeting SHALL still display a random time-of-day greeting as the heading (existing behavior)
- **AND** the greeting subtitle SHALL display a random attention message incorporating the count N of due + overdue plants, picked from a pool such as: "N plants are thirsty today.", "N plants could use a drink.", "N plants are waiting for water.", "Your plants are calling — N need water.", "Time to hydrate! N plants are due."
- **AND** if N is 1, the messages SHALL use singular form, e.g. "1 plant is thirsty today.", "1 plant could use a drink."

#### Scenario: Greeting subtitle when all ok

- **WHEN** all plants have `watering_status` of `ok`
- **THEN** the greeting SHALL display a random time-of-day greeting as the heading (existing behavior)
- **AND** the greeting subtitle SHALL display a random time-of-day subtitle (existing behavior)

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, light needs, and notes
- **AND** displays the care journal section below the watering card

#### Scenario: Care journal delete control

- **WHEN** the plant detail view is rendered
- **THEN** each care journal entry shows a delete control

#### Scenario: Edit action

- **WHEN** the user clicks the edit button on the detail view
- **THEN** the app navigates to `/plants/1/edit`

#### Scenario: Delete action

- **WHEN** the user confirms deletion on the detail view
- **THEN** the app sends `DELETE /api/plants/1`
- **AND** navigates back to the dashboard

#### Scenario: Plant not found

- **WHEN** the user navigates to `/plants/999`
- **AND** the API returns 404
- **THEN** the page displays a "Plant not found" message

#### Scenario: Widescreen detail layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the detail page max-width SHALL be 960px (increased from 800px)
- **AND** the hero photo/icon SHALL be 100px (increased from 80px)

### Requirement: Add Plant Form

The route `/plants/new` SHALL display a form to create a new plant.

#### Scenario: Form submitted

- **WHEN** the user fills in the form and clicks Save
- **THEN** the app sends `POST /api/plants` with the form data
- **AND** navigates to the new plant's detail view on success

#### Scenario: Name required

- **WHEN** the user tries to save without entering a name
- **THEN** the form shows a validation message and does not submit

#### Scenario: Icon picker

- **WHEN** the user selects an icon from the Noto emoji picker
- **THEN** the selected icon is shown on the form and sent with the request

#### Scenario: Location selection

- **WHEN** the form loads
- **THEN** location chips are populated from `GET /api/locations`
- **AND** the user can select an existing location or create a new one

#### Scenario: Watering interval

- **WHEN** the user selects a preset (3d, 7d, 14d, 30d) or uses the custom stepper
- **THEN** the watering interval is set accordingly

### Requirement: Edit Plant Form

The route `/plants/[id]/edit` SHALL display a pre-filled form to update an existing plant.

#### Scenario: Form pre-filled

- **WHEN** the user navigates to `/plants/1/edit`
- **THEN** the form is populated with the plant's current values

#### Scenario: Form submitted

- **WHEN** the user modifies fields and clicks Save
- **THEN** the app sends `PUT /api/plants/1` with the updated data
- **AND** navigates to the plant's detail view on success

### Requirement: API Client

A shared API client module SHALL provide typed functions for all plant and location API calls, handling JSON serialization and error responses.

#### Scenario: API error

- **WHEN** an API call returns an error status
- **THEN** the client extracts the error message and surfaces it to the calling component

### Requirement: Photo Display on Dashboard

The dashboard plant cards SHALL display the plant's photo when available, falling back to the emoji icon.

#### Scenario: Plant has photo

- **WHEN** a plant has a `photo_url`
- **THEN** the dashboard card shows the photo (circular, cover fit) instead of the emoji icon

#### Scenario: Plant has no photo

- **WHEN** a plant has no `photo_url`
- **THEN** the dashboard card shows the Noto emoji icon as before

### Requirement: Photo Display on Detail View

The plant detail hero section SHALL display the plant's photo when available.

#### Scenario: Plant has photo

- **WHEN** a plant has a `photo_url`
- **THEN** the detail hero shows the photo instead of the emoji icon

#### Scenario: Plant has no photo

- **WHEN** a plant has no `photo_url`
- **THEN** the detail hero shows the Noto emoji icon as before

### Requirement: Photo Section in Form

The plant add/edit form SHALL include a photo section for uploading and managing photos.

#### Scenario: Upload photo on new plant

- **WHEN** the user selects a photo file in the add form
- **THEN** a preview of the photo is shown
- **AND** the icon picker section is hidden
- **AND** on save, the photo is uploaded after creating the plant

#### Scenario: Upload photo on edit

- **WHEN** the user selects a photo file in the edit form
- **THEN** a preview is shown and the icon picker is hidden
- **AND** on save, the photo is uploaded after updating the plant

#### Scenario: Remove existing photo

- **WHEN** the user clicks "Remove" on an existing photo in the edit form
- **THEN** the photo is deleted via the API
- **AND** the icon picker section reappears

#### Scenario: No photo selected

- **WHEN** no photo is set or selected
- **THEN** the icon picker section is visible as before

### Requirement: Dashboard Watering Status Indicators

The dashboard plant cards SHALL display a visual indicator for plants that are due or overdue for watering.

#### Scenario: Plant overdue

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `overdue`
- **THEN** a red status badge with "Overdue" is displayed on the card

#### Scenario: Plant due

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `due`
- **THEN** an amber status badge with "Due" is displayed on the card

#### Scenario: Plant ok

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `ok`
- **THEN** no watering status indicator is shown

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

### Requirement: Dashboard Needs Attention Section

The dashboard SHALL display a "Needs Attention" section between the greeting and the "All Plants" grid, showing cards for plants that are overdue or due for watering.

#### Scenario: Plants need attention

- **WHEN** the dashboard renders with one or more plants having `watering_status` of `due` or `overdue`
- **THEN** a "Needs Attention" section SHALL be displayed with an alert-triangle icon and the title "Needs Attention"
- **AND** each overdue or due plant SHALL be rendered as an attention card showing the plant's photo (or emoji icon fallback), name, status badge, and a "Water" button

#### Scenario: No plants need attention

- **WHEN** all plants have `watering_status` of `ok`
- **THEN** the "Needs Attention" section SHALL NOT be rendered

#### Scenario: No plants exist

- **WHEN** no plants exist
- **THEN** the "Needs Attention" section SHALL NOT be rendered
- **AND** the empty state SHALL display unchanged

#### Scenario: Attention card ordering

- **WHEN** multiple plants need attention
- **THEN** overdue plants SHALL appear before due plants

### Requirement: Dashboard Attention Card Water Action

Each attention card SHALL include a "Water" button that waters the plant directly from the dashboard.

#### Scenario: Water from attention card

- **WHEN** the user clicks the "Water" button on an attention card
- **THEN** a `POST /api/plants/:id/water` request SHALL be sent
- **AND** the plants store SHALL be updated with the new watering data
- **AND** if the plant's status becomes `ok`, it SHALL be removed from the "Needs Attention" section

#### Scenario: Water button loading state

- **WHEN** the user clicks the "Water" button and the request is in progress
- **THEN** the button SHALL indicate a loading state

### Requirement: Dashboard Attention Section Responsive Layout

The "Needs Attention" section SHALL adapt to the viewport width following the mockup design.

#### Scenario: Desktop and tablet layout

- **WHEN** the viewport width is > 768px
- **THEN** attention cards SHALL display in a 2-column grid
- **AND** the Water button SHALL show a droplet icon and the label "Water"

#### Scenario: Mobile layout

- **WHEN** the viewport width is <= 768px
- **THEN** attention cards SHALL stack in a single column
- **AND** the Water button SHALL show only the droplet icon (no label)
