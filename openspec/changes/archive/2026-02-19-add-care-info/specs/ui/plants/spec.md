## MODIFIED Requirements

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

### Requirement: Add Plant Form

The route `/plants/new` SHALL display a form to create a new plant.

#### Scenario: Form submitted

- **WHEN** the user fills in the form and clicks Save
- **THEN** the app sends `POST /api/plants` with the form data including any selected care info values
- **AND** navigates to the new plant's detail view on success

#### Scenario: Name required

- **WHEN** the user tries to save without entering a name
- **THEN** the form shows a validation message and does not submit

## ADDED Requirements

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
