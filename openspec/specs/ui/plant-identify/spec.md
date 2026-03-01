## Purpose

AI plant identification flow — identify section visibility, photo upload slots, loading/error/suggestion states, suggestion carousel with apply/undo, and identify API client.

## Requirements

### Requirement: Identify section visibility

The PlantForm SHALL display an "Identify Plant" section inside the Identity section (below the name and species fields), when both conditions are met: a photo is present and the AI provider is enabled. The form SHALL check AI status via `GET /api/ai/status` on mount. The identify section SHALL display a read-only thumbnail of the main photo so the user can see which image will be used for identification without scrolling back to the Media section. This thumbnail SHALL NOT have a remove button — photo management is handled exclusively in the Media section.

#### Scenario: Photo present and AI enabled

- **WHEN** the PlantForm renders with a photo (new upload or existing `photo_url`)
- **AND** `GET /api/ai/status` returns `{ "enabled": true }`
- **THEN** the identify section SHALL be visible inside the Identity section, above the name and species fields
- **AND** a read-only thumbnail of the main photo SHALL be displayed at the top of the identify section

#### Scenario: No photo present

- **WHEN** the PlantForm renders without a photo (icon mode or empty media)
- **THEN** the identify section SHALL NOT be visible

#### Scenario: AI not enabled

- **WHEN** `GET /api/ai/status` returns `{ "enabled": false }`
- **THEN** the identify section SHALL NOT be visible regardless of photo presence

#### Scenario: AI status check fails

- **WHEN** `GET /api/ai/status` returns an error
- **THEN** the identify section SHALL NOT be visible

### Requirement: Identify button

The identify section SHALL display a button labeled "Identify Plant" with a sparkle icon that triggers the identification flow.

#### Scenario: User clicks Identify Plant

- **WHEN** the user clicks the "Identify Plant" button
- **THEN** the section transitions to the loading state
- **AND** a `POST /api/ai/identify` request is sent with the plant photo(s) as multipart form data

### Requirement: Extra photo upload slots

The identify section SHALL display two optional photo upload slots below the identify button, labeled "Close-up" and "Stem / pot", for additional photos that improve identification accuracy. These photos are ephemeral and not stored on the plant.

#### Scenario: Empty extra photo slots

- **WHEN** the identify section is in idle state
- **THEN** two upload slots SHALL be displayed with dashed borders, a camera icon, and their respective labels

#### Scenario: Add extra photo

- **WHEN** the user clicks an empty extra photo slot and selects a file
- **THEN** a thumbnail preview of the selected photo SHALL replace the dashed slot
- **AND** a remove button SHALL appear on the thumbnail

#### Scenario: Remove extra photo

- **WHEN** the user clicks the remove button on a filled extra photo slot
- **THEN** the slot SHALL return to its empty dashed-border state

#### Scenario: Extra photos included in identify request

- **WHEN** the user clicks "Identify Plant" with one or more extra photos added
- **THEN** all photos (main plant photo plus extras) SHALL be sent in the multipart request

#### Scenario: Accepted file types

- **WHEN** the user selects a file for an extra photo slot
- **THEN** the file input SHALL accept `image/jpeg`, `image/png`, and `image/webp`

### Requirement: Identify loading state

The identify section SHALL display a loading state while the identification request is in progress.

#### Scenario: Loading state displayed

- **WHEN** the identify request is in progress
- **THEN** the identify button SHALL be replaced by a spinner and "Identifying..." text
- **AND** thumbnails of the submitted photos SHALL be displayed
- **AND** shimmer placeholder lines SHALL be shown below the thumbnails

#### Scenario: Loading state prevents re-submission

- **WHEN** the identify section is in the loading state
- **THEN** no additional identify requests SHALL be possible

### Requirement: Identify suggestion card

The identify section SHALL display a suggestion carousel when the AI returns results, allowing the user to browse up to 3 ranked suggestions and apply their preferred one.

#### Scenario: Suggestion carousel content

- **WHEN** the AI returns an `IdentifyResponse` with multiple suggestions
- **THEN** the section SHALL display a carousel card showing the first suggestion
- **AND** the header SHALL show "AI Suggestion" with a counter indicating the current position (e.g., "1 / 3")
- **AND** the card SHALL display the scientific name, confidence badge, common name, summary, and "Will fill" chips for the active suggestion
- **AND** "Apply to form" and "Dismiss" buttons SHALL be visible

#### Scenario: Navigate between suggestions with buttons

- **WHEN** the suggestion carousel is visible with multiple suggestions
- **THEN** left and right chevron navigation buttons SHALL be displayed
- **AND** clicking the right button SHALL advance to the next suggestion
- **AND** clicking the left button SHALL return to the previous suggestion
- **AND** navigation SHALL wrap around (last → first, first → last)

#### Scenario: Dot indicators

- **WHEN** the suggestion carousel is visible with multiple suggestions
- **THEN** dot indicators SHALL be displayed between the navigation buttons
- **AND** the active suggestion's dot SHALL be visually distinct (filled vs. outline)
- **AND** clicking a dot SHALL navigate directly to that suggestion

#### Scenario: Touch swipe navigation on mobile

- **WHEN** the user performs a horizontal swipe gesture on the suggestion card
- **AND** the swipe distance exceeds 50px
- **THEN** the carousel SHALL navigate to the next or previous suggestion based on swipe direction

#### Scenario: Single suggestion fallback

- **WHEN** the AI returns only 1 suggestion
- **THEN** no navigation controls (buttons, dots) SHALL be displayed
- **AND** the counter SHALL NOT be shown
- **AND** the card SHALL display identically to the current single-suggestion layout

#### Scenario: Will fill chips update on navigation

- **WHEN** the user navigates to a different suggestion
- **THEN** the "Will fill" chips SHALL update to reflect the care profile of the newly active suggestion

#### Scenario: Will fill chips

- **WHEN** the suggestion card is rendered
- **THEN** a chip SHALL be shown for each field from the AI result that has a valid value matching the form's allowed options
- **AND** each chip SHALL display the field label and value (e.g., "Watering (10d)", "Light (indirect)")
- **AND** fields with invalid or missing values SHALL NOT have a chip

#### Scenario: Dismiss clears all suggestions

- **WHEN** the user clicks "Dismiss"
- **THEN** all suggestions SHALL be cleared
- **AND** the identify section SHALL return to the idle state

### Requirement: Apply AI suggestion to form

Clicking "Apply to form" SHALL auto-fill the PlantForm fields from the currently active AI suggestion. The user can edit any field after applying.

#### Scenario: Fields filled from active suggestion

- **WHEN** the user clicks "Apply to form" while viewing suggestion N
- **THEN** `species` SHALL be set to suggestion N's `scientific_name`
- **AND** `name` SHALL be set to suggestion N's `common_name` only if `name` is currently empty
- **AND** `notes` SHALL be set to suggestion N's `summary` only if `notes` is currently empty
- **AND** care profile fields SHALL be set from suggestion N's `care_profile` where values are valid
- **AND** the applied state banner SHALL show the count of fields updated

#### Scenario: Invalid AI values skipped

- **WHEN** the AI returns a care profile value that does not match the form's allowed options (e.g., `light_needs: "bright"`)
- **THEN** that field SHALL NOT be modified

#### Scenario: Applied state displayed

- **WHEN** fields have been applied
- **THEN** the identify section SHALL display a success banner showing the count of fields updated and an "Undo" button

#### Scenario: Undo restores previous values

- **WHEN** the user clicks "Undo" on the applied state banner
- **THEN** all form fields SHALL be restored to their values from before the apply
- **AND** the identify section SHALL return to the idle state

### Requirement: Identify error state

The identify section SHALL display an error state when the identification request fails.

#### Scenario: Error displayed

- **WHEN** the `POST /api/ai/identify` request fails (network error, 500, 503, or other error)
- **THEN** the identify section SHALL display an error message and a "Retry" button

#### Scenario: Retry after error

- **WHEN** the user clicks "Retry"
- **THEN** the identification request SHALL be re-submitted with the same photos
- **AND** the section SHALL transition to the loading state

### Requirement: Identify API client function

The frontend API client SHALL provide an `identifyPlant` function that sends photos to the identify endpoint.

#### Scenario: Successful identification

- **WHEN** `identifyPlant(photos)` is called with an array of `File` objects
- **THEN** a `POST` request SHALL be sent to `/api/ai/identify` with multipart form data
- **AND** each file SHALL be appended under the field name `photos`
- **AND** the response SHALL be parsed as an `IdentifyResponse` containing a `suggestions` array of `IdentifyResult` entries

#### Scenario: API error

- **WHEN** the API returns a non-200 status
- **THEN** the function SHALL throw an error with the message from the response body

### Requirement: Identify existing photo on edit form

When editing a plant with an existing `photo_url` and no new photo file selected, the identify function SHALL fetch the existing photo as a blob to include it in the identify request.

#### Scenario: Existing photo fetched for identification

- **WHEN** the user clicks "Identify Plant" on the edit form
- **AND** the plant has an existing `photo_url`
- **AND** no new photo file has been selected
- **THEN** the existing photo SHALL be fetched via its URL, converted to a `File`, and included in the identify request

### Requirement: Identify section responsive layout

The identify section SHALL adapt to the viewport width.

#### Scenario: Desktop layout

- **WHEN** the viewport width is > 768px
- **THEN** extra photo slots SHALL be 88×88px
- **AND** suggestion card action buttons SHALL display side by side

#### Scenario: Mobile layout

- **WHEN** the viewport width is ≤ 768px
- **THEN** extra photo slots SHALL be 80×80px
- **AND** suggestion card action buttons SHALL stack full-width with 44px minimum touch target height
