# Spec Delta

## MODIFIED Requirements

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, and notes
- **AND** the detail view contains a "Watering" card and a "Care Info" card

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

- **WHEN** an individual care journal entry is visible, either directly in the timeline or within an expanded group
- **THEN** the entry shows a delete control

#### Scenario: Edit action

- **WHEN** the user clicks the edit button on the detail view
- **THEN** the app navigates to `/plants/1/edit`

#### Scenario: Delete action

- **WHEN** the user clicks the delete button on the detail view
- **THEN** a destructive confirmation dialog is displayed
- **AND** the dialog message includes the plant name
- **AND** deletion only proceeds when the user confirms

#### Scenario: Delete action success

- **WHEN** the user confirms deletion and the plant is deleted successfully
- **THEN** the app navigates away from the detail page
- **AND** a global toast notification is displayed on the destination page acknowledging the deletion

#### Scenario: Delete care event failure

- **WHEN** the user tries to delete a care event and the request fails
- **THEN** a global toast notification is displayed describing the failure

#### Scenario: Plant not found

- **WHEN** the user navigates to `/plants/999`
- **AND** the API returns 404
- **THEN** the page displays a "Plant not found" message

#### Scenario: Widescreen detail layout

- **WHEN** sufficient horizontal space is available
- **THEN** the detail page SHALL use a bounded, expanded content area
- **AND** the hero SHALL place a prominent, proportionate plant photo or fallback icon alongside the plant information and actions

#### Scenario: Ask AI button displayed

- **WHEN** the plant detail view is rendered
- **AND** `GET /api/ai/status` returns `{ "enabled": true }`
- **THEN** an "Ask AI" button SHALL be displayed in the hero action area with the "Water now" button
- **AND** the buttons SHALL be stacked vertically when the viewport is wider than 768px
- **AND** the buttons SHALL be side-by-side when the viewport is 768px wide or narrower
- **AND** the button SHALL be visually distinct as an AI-related action

#### Scenario: Ask AI button hidden when AI disabled

- **WHEN** `GET /api/ai/status` returns `{ "enabled": false }` or fails
- **THEN** the "Ask AI" button SHALL NOT be rendered

#### Scenario: Ask AI opens chat drawer

- **WHEN** the user clicks the "Ask AI" button
- **THEN** a chat interface SHALL open with the current plant's data

#### Scenario: Mobile action bar hidden during chat

- **WHEN** the chat interface is open on mobile (viewport <= 768px)
- **THEN** the page action bar (Back / Edit / Delete) SHALL be hidden

#### Scenario: Mobile action bar restored on chat close

- **WHEN** the chat interface is closed on mobile
- **THEN** the page action bar SHALL reappear
