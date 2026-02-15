## MODIFIED Requirements

### Requirement: Plants Dashboard

The root route (`/`) SHALL display a grid of plant cards showing each plant's icon, name, and location.

#### Scenario: Plants loaded

- **WHEN** the user navigates to `/`
- **THEN** the page fetches plants from `GET /api/plants`
- **AND** displays a card grid with each plant's Noto emoji icon, name, and location name

#### Scenario: Widescreen dashboard layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the dashboard max-width SHALL be 1400px (increased from 1200px)
- **AND** the plant cards SHALL use a full-bleed image layout (240px tall photo area)
- **AND** the card name and location SHALL float over the image via a bottom gradient overlay
- **AND** the grid gap SHALL be 20px

### Requirement: Plant Detail View

The route `/plants/[id]` SHALL display full plant information with edit and delete actions.

#### Scenario: Widescreen detail layout

- **WHEN** the viewport width is >= 1280px
- **THEN** the detail page max-width SHALL be 960px (increased from 800px)
- **AND** the hero photo/icon SHALL be 100px (increased from 80px)
