## MODIFIED Requirements

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit, delete, and care actions.

#### Scenario: Plant displayed

- **WHEN** the user navigates to `/plants/1`
- **THEN** the page fetches the plant from `GET /api/plants/1`
- **AND** displays icon, name, species, location, watering interval, light needs, and notes
- **AND** displays the care journal section below the watering card

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

#### Scenario: Widescreen detail layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the detail page max-width SHALL be 960px (increased from 800px)
- **AND** the hero photo/icon SHALL be 100px (increased from 80px)
