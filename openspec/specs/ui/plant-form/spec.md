## Purpose

Plant add/edit forms — form structure, validation, icon picker, location selection, watering interval, photo upload, and care info toggle selectors.

## Requirements

### Requirement: Add Plant Form

The route `/plants/new` SHALL display a form to create a new plant.

#### Scenario: Form submitted without photo

- **WHEN** the user fills in the form without selecting a photo and clicks Save
- **THEN** the app sends `POST /api/plants` with the form data including any selected care info values
- **AND** navigates to the new plant's detail view on success

#### Scenario: Form submitted with photo

- **WHEN** the user fills in the form, selects a photo, and clicks Save
- **THEN** the app sends `POST /api/plants` with the form data
- **AND** uploads the selected photo after plant creation
- **AND** navigates to the new plant's detail view only after the photo upload succeeds

#### Scenario: Create request fails

- **WHEN** the create request fails after the user clicks Save
- **THEN** the user remains on the form
- **AND** a global toast notification is displayed describing the submission failure

#### Scenario: Photo upload fails after create

- **WHEN** the plant create request succeeds but the selected photo upload fails
- **THEN** the user remains on the form
- **AND** the save flow is treated as incomplete
- **AND** a global toast notification is displayed using the same failure pattern as other save failures

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

- **WHEN** the user selects a preset (3d, 5d, 7d, 14d) or uses the custom stepper
- **THEN** the watering interval is set accordingly

### Requirement: Edit Plant Form

The route `/plants/[id]/edit` SHALL display a pre-filled form to update an existing plant.

#### Scenario: Form pre-filled

- **WHEN** the user navigates to `/plants/1/edit`
- **THEN** the form is populated with the plant's current values

#### Scenario: Form submitted without new photo

- **WHEN** the user modifies fields without selecting a new photo and clicks Save
- **THEN** the app sends `PUT /api/plants/{id}` with the updated data
- **AND** navigates to the plant's detail view on success

#### Scenario: Form submitted with new photo

- **WHEN** the user modifies fields, selects a new photo, and clicks Save
- **THEN** the app sends `PUT /api/plants/{id}` with the updated data
- **AND** uploads the selected photo after the plant update succeeds
- **AND** navigates to the plant's detail view only after the photo upload succeeds

#### Scenario: Update request fails

- **WHEN** the update request fails after the user clicks Save
- **THEN** the user remains on the form
- **AND** a global toast notification is displayed describing the submission failure

#### Scenario: Photo upload fails after update

- **WHEN** the plant update request succeeds but the selected photo upload fails
- **THEN** the user remains on the form
- **AND** the save flow is treated as incomplete
- **AND** a global toast notification is displayed using the same failure pattern as other save failures

### Requirement: Photo Section in Form

The plant add/edit form SHALL include a photo section for uploading and managing photos.

#### Scenario: Upload photo on new plant

- **WHEN** the user selects a photo file in the add form
- **THEN** a preview of the photo is shown
- **AND** the icon picker section is hidden
- **AND** on save, the photo is uploaded after creating the plant
- **AND** the save flow is not considered complete until that upload succeeds

#### Scenario: Upload photo on edit

- **WHEN** the user selects a photo file in the edit form
- **THEN** a preview is shown and the icon picker is hidden
- **AND** on save, the photo is uploaded after updating the plant
- **AND** the save flow is not considered complete until that upload succeeds

#### Scenario: Remove existing photo

- **WHEN** the user clicks "Remove" on an existing photo in the edit form
- **THEN** the photo is deleted via the API
- **AND** the icon picker section reappears

#### Scenario: No photo selected

- **WHEN** no photo is set or selected
- **THEN** the icon picker section is visible as before

### Requirement: Care Info Form Section

The plant add/edit form SHALL include a "Care Info (optional)" section between "Light needs" and "Notes", containing toggle-button selectors for `difficulty`, `pet_safety`, `growth_speed`, and `soil_type`.

#### Scenario: Section rendering

- **WHEN** the plant form is rendered
- **THEN** a "Care Info" section SHALL appear with the label "(optional)" to indicate all fields are optional
- **AND** the section SHALL contain five labeled sub-groups: "Difficulty", "Pet safety", "Growth speed", "Soil type", and "Soil moisture"
- **AND** each sub-group SHALL render its allowed values as toggle buttons in the same visual style as the light needs selector

#### Scenario: No default selection

- **WHEN** the form loads for a new plant
- **THEN** no toggle button in the Care Info section SHALL be active (all fields start as null)

#### Scenario: Select a value

- **WHEN** the user taps a toggle button (e.g., "Easy" under Difficulty)
- **THEN** that button becomes active and the corresponding field is set to that value

#### Scenario: Deselect a value

- **WHEN** the user taps an already-active toggle button
- **THEN** the button becomes inactive and the corresponding field is cleared back to null

#### Scenario: Edit form pre-filled

- **WHEN** the edit form loads for a plant with `difficulty` = `easy` and `pet_safety` = `toxic`
- **THEN** the "Easy" button under Difficulty and the "Toxic" button under Pet safety SHALL be active
- **AND** Growth speed and Soil type SHALL have no active buttons

#### Scenario: Care info included in form submission

- **WHEN** the user submits the form with Difficulty set to "easy" and Pet safety set to "toxic"
- **THEN** the form data SHALL include `difficulty: "easy"` and `pet_safety: "toxic"`
- **AND** `growth_speed` and `soil_type` SHALL be omitted or sent as null

### Requirement: Media section upload hint

The photo upload placeholder in the Media section SHALL mention that a photo is needed for AI plant identification.

#### Scenario: Empty photo upload area

- **WHEN** the media section shows the empty photo upload dropzone
- **THEN** a hint SHALL be displayed indicating that a photo is also needed to identify a plant
