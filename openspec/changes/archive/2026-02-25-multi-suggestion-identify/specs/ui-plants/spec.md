## MODIFIED Requirements

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

#### Scenario: Undo restores previous values

- **WHEN** the user clicks "Undo" on the applied state banner
- **THEN** all form fields SHALL be restored to their values from before the apply
- **AND** the identify section SHALL return to the idle state