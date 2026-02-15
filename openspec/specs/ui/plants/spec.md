## Purpose

Plant UI screens — dashboard with card grid and photo display, plant detail view with photo hero, add/edit forms with photo upload and interactive controls.

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

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit and delete actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, light needs, and notes

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
