## ADDED Requirements

### Requirement: Dashboard Needs Attention Section

The dashboard SHALL display a "Needs Attention" section between the greeting and the "All Plants" grid, showing cards for plants that are overdue or due for watering.

#### Scenario: Plants need attention

- **WHEN** the dashboard renders with one or more plants having `watering_status` of `due` or `overdue`
- **THEN** a "Needs Attention" section SHALL be displayed with an alert-triangle icon and the title "Needs Attention"
- **AND** each overdue or due plant SHALL be rendered as an attention card showing the plant's photo (or emoji icon fallback), name, status badge, and a "Water" button

#### Scenario: No plants need attention

- **WHEN** all plants have `watering_status` of `ok`
- **THEN** the "Needs Attention" section SHALL NOT be rendered

#### Scenario: No plants exist

- **WHEN** no plants exist
- **THEN** the "Needs Attention" section SHALL NOT be rendered
- **AND** the empty state SHALL display unchanged

#### Scenario: Attention card ordering

- **WHEN** multiple plants need attention
- **THEN** overdue plants SHALL appear before due plants

### Requirement: Dashboard Attention Card Water Action

Each attention card SHALL include a "Water" button that waters the plant directly from the dashboard.

#### Scenario: Water from attention card

- **WHEN** the user clicks the "Water" button on an attention card
- **THEN** a `POST /api/plants/:id/water` request SHALL be sent
- **AND** the plants store SHALL be updated with the new watering data
- **AND** if the plant's status becomes `ok`, it SHALL be removed from the "Needs Attention" section

#### Scenario: Water button loading state

- **WHEN** the user clicks the "Water" button and the request is in progress
- **THEN** the button SHALL indicate a loading state

### Requirement: Dashboard Attention Section Responsive Layout

The "Needs Attention" section SHALL adapt to the viewport width following the mockup design.

#### Scenario: Desktop and tablet layout

- **WHEN** the viewport width is > 768px
- **THEN** attention cards SHALL display in a 2-column grid
- **AND** the Water button SHALL show a droplet icon and the label "Water"

#### Scenario: Mobile layout

- **WHEN** the viewport width is <= 768px
- **THEN** attention cards SHALL stack in a single column
- **AND** the Water button SHALL show only the droplet icon (no label)

## MODIFIED Requirements

### Requirement: Plants Dashboard

The root route (`/`) SHALL display a grid of plant cards showing each plant's icon, name, and location.

#### Scenario: Plants loaded

- **WHEN** the user navigates to `/`
- **THEN** the page fetches plants from `GET /api/plants`
- **AND** displays a card grid with each plant's Noto emoji icon, name, and location name

#### Scenario: No plants

- **WHEN** the user navigates to `/`
- **AND** no plants exist
- **THEN** the page displays an empty state with an "Add Plant" prompt

#### Scenario: Add plant button

- **WHEN** the user clicks "Add Plant"
- **THEN** the app navigates to `/plants/new`

#### Scenario: Widescreen dashboard layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the dashboard max-width SHALL be 1400px (increased from 1200px)
- **AND** the plant cards SHALL use a full-bleed image layout (240px tall photo area)
- **AND** the card name and location SHALL float over the image via a bottom gradient overlay
- **AND** the grid gap SHALL be 20px

#### Scenario: Dynamic greeting subtitle when plants need attention

- **WHEN** one or more plants have `watering_status` of `due` or `overdue`
- **THEN** the greeting SHALL still display a random time-of-day greeting as the heading (existing behavior)
- **AND** the greeting subtitle SHALL display a random attention message incorporating the count N of due + overdue plants, picked from a pool such as: "N plants are thirsty today.", "N plants could use a drink.", "N plants are waiting for water.", "Your plants are calling — N need water.", "Time to hydrate! N plants are due."
- **AND** if N is 1, the messages SHALL use singular form, e.g. "1 plant is thirsty today.", "1 plant could use a drink."

#### Scenario: Greeting subtitle when all ok

- **WHEN** all plants have `watering_status` of `ok`
- **THEN** the greeting SHALL display a random time-of-day greeting as the heading (existing behavior)
- **AND** the greeting subtitle SHALL display a random time-of-day subtitle (existing behavior)
