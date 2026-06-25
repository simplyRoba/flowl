## ADDED Requirements

### Requirement: Care event photo in plant detail timeline

Care events with a `photo_url` SHALL display a clickable thumbnail in the plant detail timeline.

#### Scenario: Event with photo

- **WHEN** a care event in the plant detail timeline has a `photo_url`
- **THEN** a 72px rounded thumbnail (`object-fit: cover`) SHALL be displayed
- **AND** the thumbnail SHALL float to the right of the text content, with text wrapping beside it on wider viewports
- **AND** clicking the thumbnail SHALL open the PhotoLightbox with the full photo URL

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

Care events with a `photo_url` SHALL display a clickable thumbnail in the global care journal page.

#### Scenario: Event with photo

- **WHEN** a care event in the global journal has a `photo_url`
- **THEN** a 80px rounded thumbnail (`object-fit: cover`) SHALL be displayed
- **AND** the thumbnail SHALL float to the right of the text content, with the time remaining pinned to the top-right of the entry
- **AND** clicking the thumbnail SHALL open a PhotoLightbox with the full photo URL

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
