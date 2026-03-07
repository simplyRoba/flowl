## MODIFIED Requirements

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
- **AND** a visible failure message is displayed so the user understands that the plant exists but the photo did not save

### Requirement: Edit Plant Form

The route `/plants/[id]/edit` SHALL display a pre-filled form to update an existing plant.

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
- **AND** a visible failure message is displayed so the user understands that the plant changes succeeded but the photo did not save

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
