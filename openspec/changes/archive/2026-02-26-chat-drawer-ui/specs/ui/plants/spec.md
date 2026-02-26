## MODIFIED Requirements

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, and notes
- **AND** the detail grid contains a "Watering" card and a "Care Info" card

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
