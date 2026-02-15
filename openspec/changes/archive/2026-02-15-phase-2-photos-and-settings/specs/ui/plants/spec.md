## Purpose

Plant UI screens — MODIFIED to display photos on dashboard/detail and add photo section to the form.

## Requirements

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
